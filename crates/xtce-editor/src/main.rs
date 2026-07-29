mod forms;
mod support_log;

use std::collections::{HashMap, HashSet};

use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    ActiveTheme, GlobalState, Icon, IconName, Root, Sizable, StyledExt, TitleBar, WindowExt,
    button::{Button, ButtonVariant, ButtonVariants},
    dialog::DialogButtonProps,
    h_flex,
    input::{Input, InputEvent as ComponentInputEvent, InputState},
    list::ListItem,
    menu::{AppMenuBar, DropdownMenu, PopupMenuItem},
    resizable::{h_resizable, resizable_panel},
    tooltip::Tooltip,
    tree::{TreeEvent, TreeItem, TreeState, tree},
    v_flex,
};

use forms::ElementForms;

actions!(
    xtce_editor,
    [
        NewDocument,
        OpenDocument,
        SaveDocument,
        ExportApplicationLog
    ]
);

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum ElementKind {
    SpaceSystem,
    TelemetryMetaData,
    TelemetryParameterTypeSet,
    TelemetryParameterType(usize),
    TelemetryParameterSet,
    TelemetryParameter(usize),
    ContainerSet,
    SequenceContainer(usize),
    MessageSet,
    Message(usize),
    TelemetryStreamSet,
    TelemetryFixedFrameStream(usize),
    TelemetryVariableFrameStream(usize),
    TelemetryCustomStream(usize),
    TelemetryAlgorithmSet,
    TelemetryCustomAlgorithm(usize),
    TelemetryMathAlgorithm(usize),
    CommandMetaData,
    CommandParameterTypeSet,
    CommandParameterType(usize),
    CommandParameterSet,
    CommandParameter(usize),
    ArgumentTypeSet,
    ArgumentType(usize),
    MetaCommandSet,
    MetaCommand(usize),
    CommandContainerSet,
    CommandContainer(usize),
    CommandStreamSet,
    CommandFixedFrameStream(usize),
    CommandVariableFrameStream(usize),
    CommandCustomStream(usize),
    CommandAlgorithmSet,
    CommandCustomAlgorithm(usize),
    CommandMathAlgorithm(usize),
    ServiceSet,
    Service(usize),
}

impl ElementKind {
    fn label(self) -> &'static str {
        match self {
            Self::SpaceSystem => "SpaceSystem",
            Self::TelemetryMetaData => "TelemetryMetaData",
            Self::TelemetryParameterTypeSet | Self::CommandParameterTypeSet => "ParameterTypeSet",
            Self::TelemetryParameterType(_) | Self::CommandParameterType(_) => "ParameterType",
            Self::TelemetryParameterSet | Self::CommandParameterSet => "ParameterSet",
            Self::TelemetryParameter(_) | Self::CommandParameter(_) => "Parameter",
            Self::ContainerSet => "ContainerSet",
            Self::SequenceContainer(_) => "SequenceContainer",
            Self::MessageSet => "MessageSet",
            Self::Message(_) => "Message",
            Self::TelemetryStreamSet | Self::CommandStreamSet => "StreamSet",
            Self::TelemetryFixedFrameStream(_) | Self::CommandFixedFrameStream(_) => {
                "FixedFrameStream"
            }
            Self::TelemetryVariableFrameStream(_) | Self::CommandVariableFrameStream(_) => {
                "VariableFrameStream"
            }
            Self::TelemetryCustomStream(_) | Self::CommandCustomStream(_) => "CustomStream",
            Self::TelemetryAlgorithmSet | Self::CommandAlgorithmSet => "AlgorithmSet",
            Self::TelemetryCustomAlgorithm(_) | Self::CommandCustomAlgorithm(_) => {
                "CustomAlgorithm"
            }
            Self::TelemetryMathAlgorithm(_) | Self::CommandMathAlgorithm(_) => "MathAlgorithm",
            Self::CommandMetaData => "CommandMetaData",
            Self::ArgumentTypeSet => "ArgumentTypeSet",
            Self::ArgumentType(_) => "ArgumentType",
            Self::MetaCommandSet => "MetaCommandSet",
            Self::MetaCommand(_) => "MetaCommand",
            Self::CommandContainerSet => "CommandContainerSet",
            Self::CommandContainer(_) => "CommandContainer",
            Self::ServiceSet => "ServiceSet",
            Self::Service(_) => "Service",
        }
    }

    fn can_add_child(self) -> bool {
        matches!(
            self,
            Self::TelemetryParameterTypeSet
                | Self::CommandParameterTypeSet
                | Self::TelemetryParameterSet
                | Self::ContainerSet
                | Self::MessageSet
                | Self::TelemetryStreamSet
                | Self::CommandStreamSet
                | Self::TelemetryAlgorithmSet
                | Self::CommandAlgorithmSet
                | Self::CommandParameterSet
                | Self::ArgumentTypeSet
                | Self::MetaCommandSet
                | Self::CommandContainerSet
                | Self::ServiceSet
        )
    }

    fn can_delete(self) -> bool {
        matches!(
            self,
            Self::TelemetryParameterType(_)
                | Self::CommandParameterType(_)
                | Self::TelemetryParameter(_)
                | Self::CommandParameter(_)
                | Self::SequenceContainer(_)
                | Self::Message(_)
                | Self::TelemetryFixedFrameStream(_)
                | Self::TelemetryVariableFrameStream(_)
                | Self::TelemetryCustomStream(_)
                | Self::CommandFixedFrameStream(_)
                | Self::CommandVariableFrameStream(_)
                | Self::CommandCustomStream(_)
                | Self::TelemetryCustomAlgorithm(_)
                | Self::TelemetryMathAlgorithm(_)
                | Self::CommandCustomAlgorithm(_)
                | Self::CommandMathAlgorithm(_)
                | Self::ArgumentType(_)
                | Self::MetaCommand(_)
                | Self::CommandContainer(_)
                | Self::Service(_)
        )
    }

    fn parent_set(self) -> Option<Self> {
        match self {
            Self::TelemetryParameterType(_) => Some(Self::TelemetryParameterTypeSet),
            Self::CommandParameterType(_) => Some(Self::CommandParameterTypeSet),
            Self::TelemetryParameter(_) => Some(Self::TelemetryParameterSet),
            Self::CommandParameter(_) => Some(Self::CommandParameterSet),
            Self::SequenceContainer(_) => Some(Self::ContainerSet),
            Self::Message(_) => Some(Self::MessageSet),
            Self::TelemetryFixedFrameStream(_)
            | Self::TelemetryVariableFrameStream(_)
            | Self::TelemetryCustomStream(_) => Some(Self::TelemetryStreamSet),
            Self::CommandFixedFrameStream(_)
            | Self::CommandVariableFrameStream(_)
            | Self::CommandCustomStream(_) => Some(Self::CommandStreamSet),
            Self::TelemetryCustomAlgorithm(_) | Self::TelemetryMathAlgorithm(_) => {
                Some(Self::TelemetryAlgorithmSet)
            }
            Self::CommandCustomAlgorithm(_) | Self::CommandMathAlgorithm(_) => {
                Some(Self::CommandAlgorithmSet)
            }
            Self::ArgumentType(_) => Some(Self::ArgumentTypeSet),
            Self::MetaCommand(_) => Some(Self::MetaCommandSet),
            Self::CommandContainer(_) => Some(Self::CommandContainerSet),
            Self::Service(_) => Some(Self::ServiceSet),
            _ => None,
        }
    }

    fn breadcrumb_directories(self) -> &'static [&'static str] {
        match self {
            Self::SpaceSystem
            | Self::TelemetryMetaData
            | Self::CommandMetaData
            | Self::ServiceSet => &[],
            Self::TelemetryParameterTypeSet
            | Self::TelemetryParameterSet
            | Self::ContainerSet
            | Self::MessageSet
            | Self::TelemetryStreamSet
            | Self::TelemetryAlgorithmSet => &["TelemetryMetaData"],
            Self::TelemetryParameterType(_) => &["TelemetryMetaData", "ParameterTypeSet"],
            Self::TelemetryParameter(_) => &["TelemetryMetaData", "ParameterSet"],
            Self::SequenceContainer(_) => &["TelemetryMetaData", "ContainerSet"],
            Self::Message(_) => &["TelemetryMetaData", "MessageSet"],
            Self::TelemetryFixedFrameStream(_)
            | Self::TelemetryVariableFrameStream(_)
            | Self::TelemetryCustomStream(_) => &["TelemetryMetaData", "StreamSet"],
            Self::TelemetryCustomAlgorithm(_) | Self::TelemetryMathAlgorithm(_) => {
                &["TelemetryMetaData", "AlgorithmSet"]
            }
            Self::CommandParameterTypeSet
            | Self::CommandParameterSet
            | Self::ArgumentTypeSet
            | Self::MetaCommandSet
            | Self::CommandContainerSet
            | Self::CommandStreamSet
            | Self::CommandAlgorithmSet => &["CommandMetaData"],
            Self::CommandParameterType(_) => &["CommandMetaData", "ParameterTypeSet"],
            Self::CommandParameter(_) => &["CommandMetaData", "ParameterSet"],
            Self::ArgumentType(_) => &["CommandMetaData", "ArgumentTypeSet"],
            Self::MetaCommand(_) => &["CommandMetaData", "MetaCommandSet"],
            Self::CommandContainer(_) => &["CommandMetaData", "CommandContainerSet"],
            Self::CommandFixedFrameStream(_)
            | Self::CommandVariableFrameStream(_)
            | Self::CommandCustomStream(_) => &["CommandMetaData", "StreamSet"],
            Self::CommandCustomAlgorithm(_) | Self::CommandMathAlgorithm(_) => {
                &["CommandMetaData", "AlgorithmSet"]
            }
            Self::Service(_) => &["ServiceSet"],
        }
    }

    fn is_directory_only(self) -> bool {
        matches!(
            self,
            Self::TelemetryMetaData
                | Self::TelemetryParameterTypeSet
                | Self::TelemetryParameterSet
                | Self::ContainerSet
                | Self::TelemetryStreamSet
                | Self::CommandMetaData
                | Self::CommandParameterTypeSet
                | Self::CommandParameterSet
                | Self::ArgumentTypeSet
                | Self::MetaCommandSet
                | Self::CommandContainerSet
                | Self::CommandStreamSet
                | Self::ServiceSet
        )
    }

    fn uses_folder_icon(self) -> bool {
        matches!(
            self,
            Self::TelemetryParameterTypeSet
                | Self::TelemetryParameterSet
                | Self::ContainerSet
                | Self::CommandParameterTypeSet
                | Self::CommandParameterSet
                | Self::ArgumentTypeSet
                | Self::MetaCommandSet
                | Self::CommandContainerSet
                | Self::TelemetryAlgorithmSet
                | Self::CommandAlgorithmSet
                | Self::ServiceSet
        )
    }

    fn tree_icon(self, expanded: bool) -> IconName {
        match self {
            Self::SpaceSystem => IconName::Globe,
            Self::TelemetryMetaData => IconName::ChartPie,
            Self::CommandMetaData | Self::MetaCommand(_) => IconName::SquareTerminal,
            Self::TelemetryParameterType(_) | Self::CommandParameterType(_) => IconName::Settings2,
            Self::TelemetryParameter(_) | Self::CommandParameter(_) => IconName::Asterisk,
            Self::SequenceContainer(_) | Self::CommandContainerSet | Self::CommandContainer(_) => {
                IconName::Frame
            }
            Self::MessageSet | Self::Message(_) => IconName::Inbox,
            Self::TelemetryStreamSet
            | Self::CommandStreamSet
            | Self::TelemetryFixedFrameStream(_)
            | Self::CommandFixedFrameStream(_)
            | Self::TelemetryVariableFrameStream(_)
            | Self::CommandVariableFrameStream(_)
            | Self::TelemetryCustomStream(_)
            | Self::CommandCustomStream(_) => IconName::GalleryVerticalEnd,
            Self::TelemetryAlgorithmSet
            | Self::CommandAlgorithmSet
            | Self::TelemetryCustomAlgorithm(_)
            | Self::CommandCustomAlgorithm(_)
            | Self::TelemetryMathAlgorithm(_)
            | Self::CommandMathAlgorithm(_) => IconName::Bot,
            Self::ArgumentType(_) => IconName::CaseSensitive,
            kind if kind.uses_folder_icon() => {
                if expanded {
                    IconName::FolderOpen
                } else {
                    IconName::Folder
                }
            }
            _ => IconName::File,
        }
    }
}

#[derive(Clone, Copy)]
enum StreamChildKind {
    Fixed,
    Variable,
    Custom,
}

#[derive(Clone, Copy)]
enum AlgorithmChildKind {
    Custom,
    Math,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct ElementSelection {
    system_path: Vec<usize>,
    kind: ElementKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BreadcrumbItem {
    label: String,
    system_path: Option<Vec<usize>>,
}

fn breadcrumb_items(
    root: &xtce::SpaceSystem,
    selection: &ElementSelection,
    current_label: String,
) -> Vec<BreadcrumbItem> {
    let mut items = Vec::new();
    let mut system = root;
    let mut path = Vec::new();
    items.push(BreadcrumbItem {
        label: system.name.clone(),
        system_path: Some(path.clone()),
    });

    for &index in &selection.system_path {
        system = &system.space_system[index];
        path.push(index);
        items.push(BreadcrumbItem {
            label: system.name.clone(),
            system_path: Some(path.clone()),
        });
    }

    if selection.kind == ElementKind::SpaceSystem {
        if let Some(current) = items.last_mut() {
            current.label = current_label;
        }
        return items;
    }

    items.extend(
        selection
            .kind
            .breadcrumb_directories()
            .iter()
            .map(|label| BreadcrumbItem {
                label: (*label).to_owned(),
                system_path: None,
            }),
    );
    items.push(BreadcrumbItem {
        label: current_label,
        system_path: None,
    });
    items
}

fn abbreviated_breadcrumb_label(label: &str) -> String {
    const MAX_CHARACTERS: usize = 28;
    let mut characters = label.chars();
    let prefix: String = characters.by_ref().take(MAX_CHARACTERS).collect();
    if characters.next().is_some() {
        format!("{prefix}…")
    } else {
        prefix
    }
}

#[derive(Clone)]
pub(crate) struct DraggedTelemetryParameter {
    reference: String,
}

struct ParameterDragPreview {
    reference: String,
}

impl Render for ParameterDragPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .gap_2()
            .px_3()
            .py_2()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .shadow_md()
            .text_sm()
            .child(Icon::new(IconName::Asterisk).small())
            .child(self.reference.clone())
    }
}

#[derive(Clone)]
struct TreeNode {
    selection: ElementSelection,
    label: String,
    drag_parameter_reference: Option<String>,
    level: usize,
    has_children: bool,
    can_add_telemetry_metadata: bool,
    can_add_command_metadata: bool,
    can_add_service_set: bool,
}

struct XtceDocument {
    root: xtce::SpaceSystem,
    selection: ElementSelection,
    file_name: String,
}

struct EditorChrome {
    app_menu_bar: Entity<AppMenuBar>,
}

struct ElementTree {
    search_input: Entity<InputState>,
    collapsed: HashSet<ElementSelection>,
    filter_collapsed: HashSet<ElementSelection>,
    tree_state: Entity<TreeState>,
    nodes: HashMap<SharedString, TreeNode>,
    ordered_nodes: Vec<TreeNode>,
    node_count: usize,
    filter_match_count: Option<usize>,
    selection_outside_filter: bool,
    hidden_selection_ancestor: Option<SharedString>,
    file_name: String,
    selection: ElementSelection,
    editor: WeakEntity<XtceEditor>,
    _subscriptions: Vec<Subscription>,
}

struct ElementInspector {
    forms: ElementForms,
}

struct XtceEditor {
    document: XtceDocument,
    chrome: EditorChrome,
    tree: Entity<ElementTree>,
    inspector: ElementInspector,
}

impl XtceEditor {
    fn new(document: XtceDocument, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let root = &document.root;
        let inspector = ElementInspector {
            forms: ElementForms::new(root, window, cx),
        };
        let tree = ElementTree::new(
            root,
            document.selection.clone(),
            document.file_name.clone(),
            cx.entity().downgrade(),
            window,
            cx,
        );

        Self {
            document,
            chrome: EditorChrome {
                app_menu_bar: AppMenuBar::new(cx),
            },
            tree,
            inspector,
        }
    }

    fn load_selected_element(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let kind = self.document.selection.kind;
        self.inspector
            .forms
            .load(kind, self.document.selected_system(), window, cx);
        self.refresh_tree(cx);
        cx.notify();
    }

    fn select_summary_element(
        &mut self,
        kind: ElementKind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.save_selected_element(cx);
        self.document.selection.kind = kind;
        self.load_selected_element(window, cx);
    }

    fn save_selected_element(&mut self, cx: &mut Context<Self>) {
        let kind = self.document.selection.kind;
        let path = self.document.selection.system_path.clone();
        let system = XtceDocument::system_at_path_mut(&mut self.document.root, &path);
        self.inspector.forms.apply_to(kind, system, cx);
        self.refresh_tree(cx);
        cx.notify();
    }

    fn refresh_tree(&mut self, cx: &mut Context<Self>) {
        let mut nodes = Vec::new();
        XtceDocument::collect_tree_nodes(&self.document.root, &mut Vec::new(), 0, &mut nodes);
        if let Some(draft_name) = self.inspector.forms.draft_name(
            self.document.selection.kind,
            self.document.selected_system(),
            cx,
        ) && let Some(selected) = nodes
            .iter_mut()
            .find(|node| node.selection == self.document.selection)
        {
            selected.label = draft_name.clone();
            if matches!(
                self.document.selection.kind,
                ElementKind::TelemetryParameter(_)
            ) {
                selected.drag_parameter_reference = Some(draft_name);
            }
        }
        let selection = self.document.selection.clone();
        let file_name = self.document.file_name.clone();
        self.tree.update(cx, |tree, cx| {
            tree.load_nodes(nodes, selection, file_name, cx);
        });
    }

    fn save_document(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        support_log::event("INFO", "save requested");
        self.save_selected_element(cx);
        let xml = match XtceDocument::serialize(&self.document.root) {
            Ok(xml) => xml,
            Err(error) => {
                support_log::event("ERROR", &format!("XTCE serialization failed: {error}"));
                window.push_notification(format!("Could not encode XTCE XML: {error}"), cx);
                return;
            }
        };
        let directory = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let receiver = cx.prompt_for_new_path(&directory, Some(&self.document.file_name));
        cx.spawn_in(window, async move |this, window| {
            let path = match receiver.await {
                Ok(Ok(Some(path))) => path,
                Ok(Ok(None)) => {
                    support_log::event("INFO", "save cancelled");
                    return;
                }
                Ok(Err(error)) => {
                    support_log::event("ERROR", &format!("save file picker failed: {error}"));
                    return;
                }
                Err(error) => {
                    support_log::event("ERROR", &format!("save file picker interrupted: {error}"));
                    return;
                }
            };
            let result = std::fs::write(&path, xml);
            let file_name = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned());
            _ = this.update_in(window, |this, window, cx| {
                let message = match result {
                    Ok(()) => {
                        support_log::event("INFO", &format!("saved {}", path.display()));
                        if let Some(file_name) = file_name {
                            this.document.file_name = file_name;
                        }
                        this.refresh_tree(cx);
                        format!("Saved {}", path.display())
                    }
                    Err(error) => {
                        support_log::event(
                            "ERROR",
                            &format!("could not save {}: {error}", path.display()),
                        );
                        format!("Could not save {}: {error}", path.display())
                    }
                };
                window.push_notification(message, cx);
            });
        })
        .detach();
    }

    fn open_document(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        support_log::event("INFO", "open requested");
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Open an XTCE XML file".into()),
        });
        cx.spawn_in(window, async move |this, window| {
            let result = match receiver.await {
                Ok(Ok(Some(paths))) => paths
                    .into_iter()
                    .next()
                    .map(|path| {
                        let document = XtceDocument::read(&path)?;
                        Ok((path, document))
                    })
                    .transpose(),
                Ok(Ok(None)) => Ok(None),
                Ok(Err(error)) => Err(format!("Could not open the file picker: {error}")),
                Err(error) => Err(format!("The file picker was interrupted: {error}")),
            };

            _ = this.update_in(window, |this, window, cx| match result {
                Ok(Some((path, document))) => {
                    support_log::event("INFO", &format!("opened {}", path.display()));
                    this.replace_document(document, window, cx);
                    window.push_notification(format!("Opened {}", path.display()), cx);
                }
                Ok(None) => support_log::event("INFO", "open cancelled"),
                Err(error) => {
                    support_log::event("ERROR", &error);
                    window.push_notification(error, cx);
                }
            });
        })
        .detach();
    }

    fn replace_document(
        &mut self,
        document: XtceDocument,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let forms = ElementForms::new(&document.root, window, cx);
        let collapsed = ElementTree::collapsed_by_default(&document.root);
        self.document = document;
        self.inspector.forms = forms;
        self.tree.update(cx, |tree, cx| {
            tree.collapsed = collapsed;
            tree.filter_collapsed.clear();
            tree.search_input
                .update(cx, |input, cx| input.set_value("", window, cx));
        });
        self.refresh_tree(cx);
        cx.notify();
    }

    fn new_document(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        support_log::event("INFO", "new document created");
        let document = XtceDocument::untitled();
        self.replace_document(document, window, cx);
        window.push_notification("Created a new XTCE document", cx);
    }

    fn export_application_log(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        support_log::event("INFO", "application log export requested");
        let directory = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let receiver = cx.prompt_for_new_path(&directory, Some("xtce-editor.log"));
        cx.spawn_in(window, async move |this, window| {
            let path = match receiver.await {
                Ok(Ok(Some(path))) => path,
                Ok(Ok(None)) => {
                    support_log::event("INFO", "application log export cancelled");
                    return;
                }
                Ok(Err(error)) => {
                    support_log::event(
                        "ERROR",
                        &format!("application log file picker failed: {error}"),
                    );
                    return;
                }
                Err(error) => {
                    support_log::event(
                        "ERROR",
                        &format!("application log file picker interrupted: {error}"),
                    );
                    return;
                }
            };
            support_log::event(
                "INFO",
                &format!("exporting application log to {}", path.display()),
            );
            let result = support_log::export_to(&path);
            _ = this.update_in(window, |_, window, cx| {
                let message = match result {
                    Ok(()) => format!("Saved application log to {}", path.display()),
                    Err(error) => {
                        support_log::event("ERROR", &error);
                        error
                    }
                };
                window.push_notification(message, cx);
            });
        })
        .detach();
    }

    fn add_tree_child(
        &mut self,
        parent: &ElementSelection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.save_selected_element(cx);
        let system = XtceDocument::system_at_path_mut(&mut self.document.root, &parent.system_path);
        let Some(kind) = XtceDocument::add_collection_item(system, parent.kind) else {
            return;
        };
        self.tree.update(cx, |tree, _| tree.expand(parent));
        self.document.selection = ElementSelection {
            system_path: parent.system_path.clone(),
            kind,
        };
        self.load_selected_element(window, cx);
    }

    fn add_stream_child(
        &mut self,
        parent: &ElementSelection,
        child_kind: StreamChildKind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.save_selected_element(cx);
        let system = XtceDocument::system_at_path_mut(&mut self.document.root, &parent.system_path);
        let Some(kind) = XtceDocument::add_stream_item(system, parent.kind, child_kind) else {
            return;
        };
        self.tree.update(cx, |tree, _| tree.expand(parent));
        self.document.selection = ElementSelection {
            system_path: parent.system_path.clone(),
            kind,
        };
        self.load_selected_element(window, cx);
    }

    fn add_algorithm_child(
        &mut self,
        parent: &ElementSelection,
        child_kind: AlgorithmChildKind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.save_selected_element(cx);
        let system = XtceDocument::system_at_path_mut(&mut self.document.root, &parent.system_path);
        let Some(kind) = XtceDocument::add_algorithm_item(system, parent.kind, child_kind) else {
            return;
        };
        self.tree.update(cx, |tree, _| tree.expand(parent));
        self.document.selection = ElementSelection {
            system_path: parent.system_path.clone(),
            kind,
        };
        self.load_selected_element(window, cx);
    }

    fn add_metadata(
        &mut self,
        parent: &ElementSelection,
        kind: ElementKind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.save_selected_element(cx);
        let system = XtceDocument::system_at_path_mut(&mut self.document.root, &parent.system_path);
        if !XtceDocument::add_metadata(system, kind) {
            return;
        }
        self.tree.update(cx, |tree, _| tree.expand(parent));
        self.document.selection = ElementSelection {
            system_path: parent.system_path.clone(),
            kind,
        };
        self.load_selected_element(window, cx);
    }

    fn add_space_system(
        &mut self,
        parent: &ElementSelection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.save_selected_element(cx);
        let system = XtceDocument::system_at_path_mut(&mut self.document.root, &parent.system_path);
        let index = XtceDocument::add_space_system(system);
        self.tree.update(cx, |tree, _| tree.expand(parent));
        let mut system_path = parent.system_path.clone();
        system_path.push(index);
        self.document.selection = ElementSelection {
            system_path,
            kind: ElementKind::SpaceSystem,
        };
        self.load_selected_element(window, cx);
    }

    fn request_delete(
        &mut self,
        selection: &ElementSelection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !selection.kind.can_delete() {
            return;
        }
        self.save_selected_element(cx);
        let Some(name) = self.document.element_name(selection) else {
            window.push_notification("The selected element no longer exists.", cx);
            return;
        };
        let references = self.document.references_to(selection, &name);
        if !references.is_empty() {
            let description = format!(
                "{} “{}” is still referenced by:\n\n{}",
                selection.kind.label(),
                name,
                references
                    .iter()
                    .map(|reference| format!("• {reference}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            window.open_alert_dialog(cx, move |alert, _, cx| {
                alert
                    .icon(Icon::new(IconName::TriangleAlert).text_color(cx.theme().danger))
                    .title("Cannot delete referenced element")
                    .description(description.clone())
                    .button_props(DialogButtonProps::default().ok_text("Close"))
            });
            return;
        }

        let editor = cx.entity().downgrade();
        let selection = selection.clone();
        let title = format!("Delete {}?", selection.kind.label());
        let description = format!(
            "“{name}” will be removed from this SpaceSystem. This action cannot be undone."
        );
        window.open_alert_dialog(cx, move |alert, _, cx| {
            let editor = editor.clone();
            let selection = selection.clone();
            alert
                .icon(Icon::new(IconName::TriangleAlert).text_color(cx.theme().danger))
                .title(title.clone())
                .description(description.clone())
                .button_props(
                    DialogButtonProps::default()
                        .ok_variant(ButtonVariant::Danger)
                        .ok_text("Delete")
                        .cancel_text("Cancel")
                        .show_cancel(true),
                )
                .on_ok(move |_, window, cx| {
                    _ = editor.update(cx, |editor, cx| {
                        editor.delete_element(&selection, window, cx);
                    });
                    true
                })
        });
    }

    fn delete_element(
        &mut self,
        selection: &ElementSelection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !XtceDocument::delete_element(&mut self.document.root, selection) {
            window.push_notification("The selected element no longer exists.", cx);
            return;
        }
        let parent = ElementSelection {
            system_path: selection.system_path.clone(),
            kind: selection
                .kind
                .parent_set()
                .expect("deletable elements have a parent set"),
        };
        self.tree.update(cx, |tree, _| tree.expand(&parent));
        self.document.selection = parent;
        self.load_selected_element(window, cx);
        window.push_notification("Element deleted.", cx);
    }
}

impl XtceDocument {
    const XTCE_1_2_NAMESPACE: &'static str = "http://www.omg.org/spec/XTCE/20180204";

    fn untitled() -> Self {
        Self {
            root: xtce::SpaceSystem::new("NewSpaceSystem"),
            selection: ElementSelection {
                system_path: Vec::new(),
                kind: ElementKind::SpaceSystem,
            },
            file_name: "untitled.xml".to_owned(),
        }
    }

    fn serialize(root: &xtce::SpaceSystem) -> Result<String, String> {
        xtce::to_string(root)
            .map(|xml| format!("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n{xml}"))
            .map_err(|error| error.to_string())
    }

    fn read(path: &std::path::Path) -> Result<Self, String> {
        let xml = std::fs::read_to_string(path)
            .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
        if xml.contains(Self::XTCE_1_2_NAMESPACE) {
            support_log::event(
                "INFO",
                &format!(
                    "applying XTCE 1.2 namespace compatibility to {}",
                    path.display()
                ),
            );
        }
        let file_name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
        Self::from_xml(&xml, file_name)
            .map_err(|error| format!("Could not open {}: {error}", path.display()))
    }

    fn from_xml(xml: &str, file_name: String) -> Result<Self, String> {
        let root = match xtce::from_str(xml) {
            Ok(root) => root,
            Err(original_error) if xml.contains(Self::XTCE_1_2_NAMESPACE) => {
                let normalized = xml.replace(Self::XTCE_1_2_NAMESPACE, xtce::XTCE_NAMESPACE);
                xtce::from_str(&normalized).map_err(|compatibility_error| {
                    format!(
                        "XTCE 1.2 compatibility conversion failed: {compatibility_error} \
                         (original error: {original_error})"
                    )
                })?
            }
            Err(error) => return Err(error.to_string()),
        };
        Ok(Self {
            root,
            selection: ElementSelection {
                system_path: Vec::new(),
                kind: ElementKind::SpaceSystem,
            },
            file_name,
        })
    }

    fn system_at_path<'a>(system: &'a xtce::SpaceSystem, path: &[usize]) -> &'a xtce::SpaceSystem {
        match path.split_first() {
            Some((&index, remaining)) => {
                Self::system_at_path(&system.space_system[index], remaining)
            }
            None => system,
        }
    }

    fn system_at_path_mut<'a>(
        system: &'a mut xtce::SpaceSystem,
        path: &[usize],
    ) -> &'a mut xtce::SpaceSystem {
        match path.split_first() {
            Some((&index, remaining)) => {
                Self::system_at_path_mut(&mut system.space_system[index], remaining)
            }
            None => system,
        }
    }

    fn selected_system(&self) -> &xtce::SpaceSystem {
        Self::system_at_path(&self.root, &self.selection.system_path)
    }

    fn element_name(&self, selection: &ElementSelection) -> Option<String> {
        let system = Self::system_at_path(&self.root, &selection.system_path);
        match selection.kind {
            ElementKind::TelemetryParameterType(index) => system
                .telemetry_meta_data
                .as_ref()?
                .parameter_type_set
                .as_ref()?
                .content
                .get(index)
                .map(Self::parameter_type_label),
            ElementKind::CommandParameterType(index) => system
                .command_meta_data
                .as_ref()?
                .parameter_type_set
                .as_ref()?
                .content
                .get(index)
                .map(Self::parameter_type_label),
            ElementKind::TelemetryParameter(index) => system
                .telemetry_meta_data
                .as_ref()?
                .parameter_set
                .as_ref()?
                .content
                .get(index)
                .map(Self::parameter_label),
            ElementKind::CommandParameter(index) => system
                .command_meta_data
                .as_ref()?
                .parameter_set
                .as_ref()?
                .content
                .get(index)
                .map(Self::parameter_label),
            ElementKind::SequenceContainer(index) => system
                .telemetry_meta_data
                .as_ref()?
                .container_set
                .as_ref()?
                .content
                .get(index)
                .map(Self::sequence_container_label),
            ElementKind::Message(index) => system
                .telemetry_meta_data
                .as_ref()?
                .message_set
                .as_ref()?
                .message
                .get(index)
                .map(|message| message.name.clone()),
            ElementKind::TelemetryFixedFrameStream(index)
            | ElementKind::TelemetryVariableFrameStream(index)
            | ElementKind::TelemetryCustomStream(index) => system
                .telemetry_meta_data
                .as_ref()?
                .stream_set
                .as_ref()?
                .content
                .get(index)
                .map(Self::stream_label),
            ElementKind::CommandFixedFrameStream(index)
            | ElementKind::CommandVariableFrameStream(index)
            | ElementKind::CommandCustomStream(index) => system
                .command_meta_data
                .as_ref()?
                .stream_set
                .as_ref()?
                .content
                .get(index)
                .map(Self::stream_label),
            ElementKind::TelemetryCustomAlgorithm(index)
            | ElementKind::TelemetryMathAlgorithm(index) => system
                .telemetry_meta_data
                .as_ref()?
                .algorithm_set
                .as_ref()?
                .content
                .get(index)
                .map(Self::algorithm_label),
            ElementKind::CommandCustomAlgorithm(index)
            | ElementKind::CommandMathAlgorithm(index) => system
                .command_meta_data
                .as_ref()?
                .algorithm_set
                .as_ref()?
                .content
                .get(index)
                .map(Self::algorithm_label),
            ElementKind::ArgumentType(index) => system
                .command_meta_data
                .as_ref()?
                .argument_type_set
                .as_ref()?
                .content
                .get(index)
                .map(Self::argument_type_label),
            ElementKind::MetaCommand(index) => system
                .command_meta_data
                .as_ref()?
                .meta_command_set
                .as_ref()?
                .content
                .get(index)
                .map(Self::meta_command_label),
            ElementKind::CommandContainer(index) => system
                .command_meta_data
                .as_ref()?
                .command_container_set
                .as_ref()?
                .command_container
                .get(index)
                .map(|container| container.name.clone()),
            ElementKind::Service(index) => system
                .service_set
                .as_ref()?
                .service
                .get(index)
                .map(|service| service.name.clone()),
            _ => None,
        }
    }

    fn references_to(&self, selection: &ElementSelection, name: &str) -> Vec<String> {
        let attribute_names: &[&str] = match selection.kind {
            ElementKind::TelemetryParameterType(_) | ElementKind::CommandParameterType(_) => {
                &["parametertyperef", "arraytyperef", "typeref"]
            }
            ElementKind::TelemetryParameter(_) | ElementKind::CommandParameter(_) => {
                &["parameterref", "outputparameterref"]
            }
            ElementKind::SequenceContainer(_) | ElementKind::CommandContainer(_) => {
                &["containerref"]
            }
            ElementKind::Message(_) => &["messageref"],
            ElementKind::TelemetryFixedFrameStream(_)
            | ElementKind::TelemetryVariableFrameStream(_)
            | ElementKind::TelemetryCustomStream(_)
            | ElementKind::CommandFixedFrameStream(_)
            | ElementKind::CommandVariableFrameStream(_)
            | ElementKind::CommandCustomStream(_) => {
                &["streamref", "encodedstreamref", "decodedstreamref"]
            }
            ElementKind::ArgumentType(_) => &["argumenttyperef", "arraytyperef", "typeref"],
            ElementKind::MetaCommand(_) => &["metacommandref"],
            _ => &[],
        };
        if attribute_names.is_empty() {
            return Vec::new();
        }
        let Ok(xml) = Self::serialize(&self.root) else {
            return Vec::new();
        };
        let Ok(document) = roxmltree::Document::parse(&xml) else {
            return Vec::new();
        };
        let mut references = Vec::new();
        for node in document.descendants().filter(|node| node.is_element()) {
            for attribute in node.attributes() {
                let attribute_name = attribute.name().to_ascii_lowercase();
                if !attribute_names.contains(&attribute_name.as_str())
                    || !Self::reference_matches(attribute.value(), name)
                {
                    continue;
                }
                let source = node
                    .ancestors()
                    .filter(|ancestor| ancestor.is_element())
                    .find_map(|ancestor| {
                        ancestor
                            .attribute("name")
                            .map(|source_name| (ancestor.tag_name().name(), source_name))
                    });
                if matches!(source, Some((_, source_name)) if source_name == name) {
                    continue;
                }
                let label = match source {
                    Some((tag, source_name)) => {
                        format!("{tag} “{source_name}” ({})", attribute.name())
                    }
                    None => format!("{} ({})", node.tag_name().name(), attribute.name()),
                };
                if !references.contains(&label) {
                    references.push(label);
                }
            }
        }
        references
    }

    fn reference_matches(reference: &str, name: &str) -> bool {
        reference == name
            || reference
                .trim_end_matches('/')
                .rsplit('/')
                .next()
                .is_some_and(|segment| segment == name)
    }

    fn delete_element(root: &mut xtce::SpaceSystem, selection: &ElementSelection) -> bool {
        let system = Self::system_at_path_mut(root, &selection.system_path);
        match selection.kind {
            ElementKind::TelemetryParameterType(index) => {
                let Some(metadata) = system.telemetry_meta_data.as_mut() else {
                    return false;
                };
                let Some(set) = metadata.parameter_type_set.as_mut() else {
                    return false;
                };
                if index >= set.content.len() {
                    return false;
                }
                set.content.remove(index);
                if set.content.is_empty() {
                    metadata.parameter_type_set = None;
                }
            }
            ElementKind::CommandParameterType(index) => {
                let Some(metadata) = system.command_meta_data.as_mut() else {
                    return false;
                };
                let Some(set) = metadata.parameter_type_set.as_mut() else {
                    return false;
                };
                if index >= set.content.len() {
                    return false;
                }
                set.content.remove(index);
                if set.content.is_empty() {
                    metadata.parameter_type_set = None;
                }
            }
            ElementKind::TelemetryParameter(index) => {
                let Some(metadata) = system.telemetry_meta_data.as_mut() else {
                    return false;
                };
                let Some(set) = metadata.parameter_set.as_mut() else {
                    return false;
                };
                if index >= set.content.len() {
                    return false;
                }
                set.content.remove(index);
                if set.content.is_empty() {
                    metadata.parameter_set = None;
                }
            }
            ElementKind::CommandParameter(index) => {
                let Some(metadata) = system.command_meta_data.as_mut() else {
                    return false;
                };
                let Some(set) = metadata.parameter_set.as_mut() else {
                    return false;
                };
                if index >= set.content.len() {
                    return false;
                }
                set.content.remove(index);
                if set.content.is_empty() {
                    metadata.parameter_set = None;
                }
            }
            ElementKind::SequenceContainer(index) => {
                let Some(metadata) = system.telemetry_meta_data.as_mut() else {
                    return false;
                };
                let Some(set) = metadata.container_set.as_mut() else {
                    return false;
                };
                if index >= set.content.len() {
                    return false;
                }
                set.content.remove(index);
                if set.content.is_empty() {
                    metadata.container_set = None;
                }
            }
            ElementKind::Message(index) => {
                let Some(metadata) = system.telemetry_meta_data.as_mut() else {
                    return false;
                };
                let Some(set) = metadata.message_set.as_mut() else {
                    return false;
                };
                if index >= set.message.len() {
                    return false;
                }
                set.message.remove(index);
                if set.message.is_empty()
                    && set.name.is_none()
                    && set.short_description.is_none()
                    && set.long_description.is_none()
                    && set.alias_set.is_none()
                    && set.ancillary_data_set.is_none()
                {
                    metadata.message_set = None;
                }
            }
            ElementKind::TelemetryFixedFrameStream(index)
            | ElementKind::TelemetryVariableFrameStream(index)
            | ElementKind::TelemetryCustomStream(index) => {
                let Some(metadata) = system.telemetry_meta_data.as_mut() else {
                    return false;
                };
                let Some(set) = metadata.stream_set.as_mut() else {
                    return false;
                };
                if index >= set.content.len() {
                    return false;
                }
                set.content.remove(index);
                if set.content.is_empty() {
                    metadata.stream_set = None;
                }
            }
            ElementKind::CommandFixedFrameStream(index)
            | ElementKind::CommandVariableFrameStream(index)
            | ElementKind::CommandCustomStream(index) => {
                let Some(metadata) = system.command_meta_data.as_mut() else {
                    return false;
                };
                let Some(set) = metadata.stream_set.as_mut() else {
                    return false;
                };
                if index >= set.content.len() {
                    return false;
                }
                set.content.remove(index);
                if set.content.is_empty() {
                    metadata.stream_set = None;
                }
            }
            ElementKind::TelemetryCustomAlgorithm(index)
            | ElementKind::TelemetryMathAlgorithm(index) => {
                let Some(metadata) = system.telemetry_meta_data.as_mut() else {
                    return false;
                };
                let Some(set) = metadata.algorithm_set.as_mut() else {
                    return false;
                };
                if index >= set.content.len() {
                    return false;
                }
                set.content.remove(index);
                if set.content.is_empty() {
                    metadata.algorithm_set = None;
                }
            }
            ElementKind::CommandCustomAlgorithm(index)
            | ElementKind::CommandMathAlgorithm(index) => {
                let Some(metadata) = system.command_meta_data.as_mut() else {
                    return false;
                };
                let Some(set) = metadata.algorithm_set.as_mut() else {
                    return false;
                };
                if index >= set.content.len() {
                    return false;
                }
                set.content.remove(index);
                if set.content.is_empty() {
                    metadata.algorithm_set = None;
                }
            }
            ElementKind::ArgumentType(index) => {
                let Some(metadata) = system.command_meta_data.as_mut() else {
                    return false;
                };
                let Some(set) = metadata.argument_type_set.as_mut() else {
                    return false;
                };
                if index >= set.content.len() {
                    return false;
                }
                set.content.remove(index);
                if set.content.is_empty() {
                    metadata.argument_type_set = None;
                }
            }
            ElementKind::MetaCommand(index) => {
                let Some(metadata) = system.command_meta_data.as_mut() else {
                    return false;
                };
                let Some(set) = metadata.meta_command_set.as_mut() else {
                    return false;
                };
                if index >= set.content.len() {
                    return false;
                }
                set.content.remove(index);
                if set.content.is_empty() {
                    metadata.meta_command_set = None;
                }
            }
            ElementKind::CommandContainer(index) => {
                let Some(metadata) = system.command_meta_data.as_mut() else {
                    return false;
                };
                let Some(set) = metadata.command_container_set.as_mut() else {
                    return false;
                };
                if index >= set.command_container.len() {
                    return false;
                }
                set.command_container.remove(index);
                if set.command_container.is_empty() {
                    metadata.command_container_set = None;
                }
            }
            ElementKind::Service(index) => {
                let Some(set) = system.service_set.as_mut() else {
                    return false;
                };
                if index >= set.service.len() {
                    return false;
                }
                set.service.remove(index);
            }
            _ => return false,
        }
        true
    }

    fn add_collection_item(
        system: &mut xtce::SpaceSystem,
        collection: ElementKind,
    ) -> Option<ElementKind> {
        match collection {
            ElementKind::TelemetryParameterTypeSet => {
                let set = system
                    .telemetry_meta_data
                    .as_mut()?
                    .parameter_type_set
                    .get_or_insert_with(|| xtce::ParameterTypeSetType {
                        content: Vec::new(),
                    });
                let index = set.content.len();
                let name = Self::next_parameter_type_name(&set.content);
                set.content.push(Self::new_parameter_type(name));
                Some(ElementKind::TelemetryParameterType(index))
            }
            ElementKind::CommandParameterTypeSet => {
                let set = system
                    .command_meta_data
                    .as_mut()?
                    .parameter_type_set
                    .get_or_insert_with(|| xtce::ParameterTypeSetType {
                        content: Vec::new(),
                    });
                let index = set.content.len();
                let name = Self::next_parameter_type_name(&set.content);
                set.content.push(Self::new_parameter_type(name));
                Some(ElementKind::CommandParameterType(index))
            }
            ElementKind::TelemetryParameterSet => {
                let type_ref = system
                    .telemetry_meta_data
                    .as_ref()
                    .and_then(|metadata| metadata.parameter_type_set.as_ref())
                    .and_then(|set| set.content.first())
                    .map(Self::parameter_type_label)
                    .unwrap_or_else(|| "ParameterType1".to_owned());
                let set = system
                    .telemetry_meta_data
                    .as_mut()?
                    .parameter_set
                    .get_or_insert_with(|| xtce::ParameterSetType {
                        content: Vec::new(),
                    });
                let index = set.content.len();
                let name = Self::next_parameter_name(&set.content);
                set.content.push(Self::new_parameter(name, type_ref));
                Some(ElementKind::TelemetryParameter(index))
            }
            ElementKind::ContainerSet => {
                let set = system
                    .telemetry_meta_data
                    .as_mut()?
                    .container_set
                    .get_or_insert_with(|| xtce::ContainerSetType {
                        content: Vec::new(),
                    });
                let index = set.content.len();
                let name = Self::next_sequence_container_name(&set.content);
                set.content.push(Self::new_sequence_container(name));
                Some(ElementKind::SequenceContainer(index))
            }
            ElementKind::MessageSet => {
                let metadata = system.telemetry_meta_data.as_mut()?;
                let set = metadata
                    .message_set
                    .get_or_insert_with(forms::message_set::default_message_set);
                let index = set.message.len();
                let name = Self::next_unique_name("Message", |candidate| {
                    set.message.iter().any(|message| message.name == candidate)
                });
                set.message.push(xtce::MessageType {
                    short_description: None,
                    name,
                    long_description: None,
                    alias_set: None,
                    ancillary_data_set: None,
                    match_criteria: xtce::MatchCriteriaType::Comparison(xtce::ComparisonType {
                        parameter_ref: String::new(),
                        instance: xtce::ComparisonType::default_instance(),
                        use_calibrated_value: xtce::ComparisonType::default_use_calibrated_value(),
                        comparison_operator: xtce::ComparisonType::default_comparison_operator(),
                        value: "0".to_owned(),
                    }),
                    container_ref: xtce::ContainerRefType {
                        container_ref: String::new(),
                    },
                });
                Some(ElementKind::Message(index))
            }
            ElementKind::TelemetryStreamSet => {
                Self::add_stream_item(system, collection, StreamChildKind::Fixed)
            }
            ElementKind::CommandStreamSet => {
                Self::add_stream_item(system, collection, StreamChildKind::Fixed)
            }
            ElementKind::TelemetryAlgorithmSet => {
                Self::add_algorithm_item(system, collection, AlgorithmChildKind::Custom)
            }
            ElementKind::CommandAlgorithmSet => {
                Self::add_algorithm_item(system, collection, AlgorithmChildKind::Custom)
            }
            ElementKind::CommandParameterSet => {
                let type_ref = system
                    .command_meta_data
                    .as_ref()
                    .and_then(|metadata| metadata.parameter_type_set.as_ref())
                    .and_then(|set| set.content.first())
                    .map(Self::parameter_type_label)
                    .unwrap_or_else(|| "ParameterType1".to_owned());
                let set = system
                    .command_meta_data
                    .as_mut()?
                    .parameter_set
                    .get_or_insert_with(|| xtce::ParameterSetType {
                        content: Vec::new(),
                    });
                let index = set.content.len();
                let name = Self::next_parameter_name(&set.content);
                set.content.push(Self::new_parameter(name, type_ref));
                Some(ElementKind::CommandParameter(index))
            }
            ElementKind::ArgumentTypeSet => {
                let set = system
                    .command_meta_data
                    .as_mut()?
                    .argument_type_set
                    .get_or_insert_with(|| xtce::ArgumentTypeSetType {
                        content: Vec::new(),
                    });
                let index = set.content.len();
                let name = Self::next_argument_type_name(&set.content);
                set.content.push(Self::new_argument_type(name));
                Some(ElementKind::ArgumentType(index))
            }
            ElementKind::MetaCommandSet => {
                let set = system
                    .command_meta_data
                    .as_mut()?
                    .meta_command_set
                    .get_or_insert_with(|| xtce::MetaCommandSetType {
                        content: Vec::new(),
                    });
                let index = set.content.len();
                let name = Self::next_meta_command_name(&set.content);
                set.content.push(Self::new_meta_command(name));
                Some(ElementKind::MetaCommand(index))
            }
            ElementKind::CommandContainerSet => {
                let set = system
                    .command_meta_data
                    .as_mut()?
                    .command_container_set
                    .get_or_insert_with(|| xtce::CommandContainerSetType {
                        command_container: Vec::new(),
                    });
                let index = set.command_container.len();
                let name = Self::next_unique_name("CommandContainer", |candidate| {
                    set.command_container
                        .iter()
                        .any(|container| container.name == candidate)
                });
                set.command_container
                    .push(Self::new_sequence_container_value(name));
                Some(ElementKind::CommandContainer(index))
            }
            ElementKind::ServiceSet => {
                let set = system.service_set.as_mut()?;
                let index = set.service.len();
                let name = Self::next_unique_name("Service", |candidate| {
                    set.service.iter().any(|service| service.name == candidate)
                });
                set.service.push(forms::service_set::default_service(name));
                Some(ElementKind::Service(index))
            }
            _ => None,
        }
    }

    fn add_stream_item(
        system: &mut xtce::SpaceSystem,
        collection: ElementKind,
        child_kind: StreamChildKind,
    ) -> Option<ElementKind> {
        let (set, telemetry) = match collection {
            ElementKind::TelemetryStreamSet => (
                system
                    .telemetry_meta_data
                    .as_mut()?
                    .stream_set
                    .get_or_insert_with(|| xtce::StreamSetType {
                        content: Vec::new(),
                    }),
                true,
            ),
            ElementKind::CommandStreamSet => (
                system
                    .command_meta_data
                    .as_mut()?
                    .stream_set
                    .get_or_insert_with(|| xtce::StreamSetType {
                        content: Vec::new(),
                    }),
                false,
            ),
            _ => return None,
        };
        let index = set.content.len();
        let prefix = match child_kind {
            StreamChildKind::Fixed => "FixedFrameStream",
            StreamChildKind::Variable => "VariableFrameStream",
            StreamChildKind::Custom => "CustomStream",
        };
        let name = Self::next_unique_name(prefix, |candidate| {
            set.content.iter().any(|stream| match stream {
                xtce::StreamSetTypeContent::FixedFrameStream(stream) => stream.name == candidate,
                xtce::StreamSetTypeContent::VariableFrameStream(stream) => stream.name == candidate,
                xtce::StreamSetTypeContent::CustomStream(stream) => stream.name == candidate,
            })
        });
        set.content.push(match child_kind {
            StreamChildKind::Fixed => forms::fixed_frame_stream::default_fixed_frame_stream(name),
            StreamChildKind::Variable => {
                forms::variable_frame_stream::default_variable_frame_stream(name)
            }
            StreamChildKind::Custom => forms::custom_stream::default_custom_stream(name),
        });
        Some(match (telemetry, child_kind) {
            (true, StreamChildKind::Fixed) => ElementKind::TelemetryFixedFrameStream(index),
            (true, StreamChildKind::Variable) => ElementKind::TelemetryVariableFrameStream(index),
            (true, StreamChildKind::Custom) => ElementKind::TelemetryCustomStream(index),
            (false, StreamChildKind::Fixed) => ElementKind::CommandFixedFrameStream(index),
            (false, StreamChildKind::Variable) => ElementKind::CommandVariableFrameStream(index),
            (false, StreamChildKind::Custom) => ElementKind::CommandCustomStream(index),
        })
    }

    fn add_algorithm_item(
        system: &mut xtce::SpaceSystem,
        collection: ElementKind,
        child_kind: AlgorithmChildKind,
    ) -> Option<ElementKind> {
        let (set, telemetry) = match collection {
            ElementKind::TelemetryAlgorithmSet => (
                system
                    .telemetry_meta_data
                    .as_mut()?
                    .algorithm_set
                    .get_or_insert_with(|| xtce::AlgorithmSetType {
                        content: Vec::new(),
                    }),
                true,
            ),
            ElementKind::CommandAlgorithmSet => (
                system
                    .command_meta_data
                    .as_mut()?
                    .algorithm_set
                    .get_or_insert_with(|| xtce::AlgorithmSetType {
                        content: Vec::new(),
                    }),
                false,
            ),
            _ => return None,
        };
        let index = set.content.len();
        let prefix = match child_kind {
            AlgorithmChildKind::Custom => "CustomAlgorithm",
            AlgorithmChildKind::Math => "MathAlgorithm",
        };
        let name = Self::next_unique_name(prefix, |candidate| {
            set.content
                .iter()
                .any(|algorithm| Self::algorithm_label(algorithm) == candidate)
        });
        set.content.push(match child_kind {
            AlgorithmChildKind::Custom => forms::custom_algorithm::default_custom_algorithm(name),
            AlgorithmChildKind::Math => forms::math_algorithm::default_math_algorithm(name),
        });
        Some(match (telemetry, child_kind) {
            (true, AlgorithmChildKind::Custom) => ElementKind::TelemetryCustomAlgorithm(index),
            (true, AlgorithmChildKind::Math) => ElementKind::TelemetryMathAlgorithm(index),
            (false, AlgorithmChildKind::Custom) => ElementKind::CommandCustomAlgorithm(index),
            (false, AlgorithmChildKind::Math) => ElementKind::CommandMathAlgorithm(index),
        })
    }

    fn add_metadata(system: &mut xtce::SpaceSystem, kind: ElementKind) -> bool {
        match kind {
            ElementKind::TelemetryMetaData if system.telemetry_meta_data.is_none() => {
                system.telemetry_meta_data = Some(xtce::TelemetryMetaDataType {
                    parameter_type_set: None,
                    parameter_set: None,
                    container_set: None,
                    message_set: None,
                    stream_set: None,
                    algorithm_set: None,
                });
                true
            }
            ElementKind::CommandMetaData if system.command_meta_data.is_none() => {
                system.command_meta_data = Some(xtce::CommandMetaDataType {
                    parameter_type_set: None,
                    parameter_set: None,
                    argument_type_set: None,
                    meta_command_set: None,
                    command_container_set: None,
                    stream_set: None,
                    algorithm_set: None,
                });
                true
            }
            ElementKind::ServiceSet if system.service_set.is_none() => {
                system.service_set = Some(xtce::ServiceSetType {
                    service: Vec::new(),
                });
                true
            }
            _ => false,
        }
    }

    fn add_space_system(system: &mut xtce::SpaceSystem) -> usize {
        let index = system.space_system.len();
        let name = Self::next_unique_name("SpaceSystem", |candidate| {
            system
                .space_system
                .iter()
                .any(|child| child.name == candidate)
        });
        system.space_system.push(xtce::SpaceSystem::new(name));
        index
    }

    fn new_parameter_type(name: String) -> xtce::ParameterTypeSetTypeContent {
        xtce::ParameterTypeSetTypeContent::StringParameterType(xtce::StringParameterType {
            short_description: None,
            name,
            base_type: None,
            initial_value: None,
            restriction_pattern: None,
            character_width: None,
            content: Vec::new(),
        })
    }

    fn new_parameter(name: String, parameter_type_ref: String) -> xtce::ParameterSetTypeContent {
        xtce::ParameterSetTypeContent::Parameter(xtce::ParameterType {
            short_description: None,
            name,
            parameter_type_ref,
            initial_value: None,
            long_description: None,
            alias_set: None,
            ancillary_data_set: None,
            parameter_properties: None,
        })
    }

    fn new_argument_type(name: String) -> xtce::ArgumentTypeSetTypeContent {
        xtce::ArgumentTypeSetTypeContent::StringArgumentType(xtce::StringArgumentType {
            short_description: None,
            name,
            base_type: None,
            initial_value: None,
            restriction_pattern: None,
            character_width: None,
            content: Vec::new(),
        })
    }

    fn new_meta_command(name: String) -> xtce::MetaCommandSetTypeContent {
        xtce::MetaCommandSetTypeContent::MetaCommand(xtce::MetaCommandType {
            short_description: None,
            name,
            abstract_: xtce::MetaCommandType::default_abstract_(),
            long_description: None,
            alias_set: None,
            ancillary_data_set: None,
            base_meta_command: None,
            system_name: None,
            argument_list: None,
            command_container: None,
            transmission_constraint_list: None,
            default_significance: None,
            context_significance_list: None,
            interlock: None,
            verifier_set: None,
            parameter_to_set_list: None,
            parameters_to_suspend_alarms_on_set: None,
        })
    }

    fn new_sequence_container(name: String) -> xtce::ContainerSetTypeContent {
        xtce::ContainerSetTypeContent::SequenceContainer(Self::new_sequence_container_value(name))
    }

    fn new_sequence_container_value(name: String) -> xtce::SequenceContainerType {
        xtce::SequenceContainerType {
            short_description: None,
            name,
            abstract_: xtce::SequenceContainerType::default_abstract_(),
            idle_pattern: xtce::SequenceContainerType::default_idle_pattern(),
            long_description: None,
            alias_set: None,
            ancillary_data_set: None,
            default_rate_in_stream: None,
            rate_in_stream_set: None,
            binary_encoding: None,
            entry_list: xtce::EntryListType {
                content: Vec::new(),
            },
            base_container: None,
        }
    }

    fn next_parameter_type_name(content: &[xtce::ParameterTypeSetTypeContent]) -> String {
        Self::next_unique_name("ParameterType", |candidate| {
            content
                .iter()
                .any(|parameter_type| Self::parameter_type_label(parameter_type) == candidate)
        })
    }

    fn next_parameter_name(content: &[xtce::ParameterSetTypeContent]) -> String {
        Self::next_unique_name("Parameter", |candidate| {
            content.iter().any(|parameter| {
                matches!(
                    parameter,
                    xtce::ParameterSetTypeContent::Parameter(parameter)
                        if parameter.name == candidate
                )
            })
        })
    }

    fn next_argument_type_name(content: &[xtce::ArgumentTypeSetTypeContent]) -> String {
        Self::next_unique_name("ArgumentType", |candidate| {
            content
                .iter()
                .any(|argument_type| Self::argument_type_label(argument_type) == candidate)
        })
    }

    fn next_meta_command_name(content: &[xtce::MetaCommandSetTypeContent]) -> String {
        Self::next_unique_name("MetaCommand", |candidate| {
            content
                .iter()
                .any(|command| Self::meta_command_label(command) == candidate)
        })
    }

    fn next_sequence_container_name(content: &[xtce::ContainerSetTypeContent]) -> String {
        Self::next_unique_name("SequenceContainer", |candidate| {
            content
                .iter()
                .any(|container| Self::sequence_container_label(container) == candidate)
        })
    }

    fn next_unique_name(prefix: &str, exists: impl Fn(&str) -> bool) -> String {
        (1..)
            .map(|index| format!("{prefix}{index}"))
            .find(|candidate| !exists(candidate))
            .expect("an available generated name should exist")
    }
}

impl XtceDocument {
    fn collect_tree_nodes(
        system: &xtce::SpaceSystem,
        path: &mut Vec<usize>,
        level: usize,
        nodes: &mut Vec<TreeNode>,
    ) {
        let has_children = system.telemetry_meta_data.is_some()
            || system.command_meta_data.is_some()
            || system.service_set.is_some()
            || !system.space_system.is_empty();
        nodes.push(TreeNode {
            selection: ElementSelection {
                system_path: path.clone(),
                kind: ElementKind::SpaceSystem,
            },
            label: system.name.clone(),
            drag_parameter_reference: None,
            level,
            has_children,
            can_add_telemetry_metadata: system.telemetry_meta_data.is_none(),
            can_add_command_metadata: system.command_meta_data.is_none(),
            can_add_service_set: system.service_set.is_none(),
        });

        let child_level = level + 1;
        if let Some(metadata) = &system.telemetry_meta_data {
            Self::push_tree_node(
                nodes,
                path,
                ElementKind::TelemetryMetaData,
                child_level,
                true,
            );
            let parameter_types = metadata
                .parameter_type_set
                .as_ref()
                .map(|set| set.content.as_slice())
                .unwrap_or_default();
            Self::push_tree_node(
                nodes,
                path,
                ElementKind::TelemetryParameterTypeSet,
                child_level + 1,
                !parameter_types.is_empty(),
            );
            for (index, parameter_type) in parameter_types.iter().enumerate() {
                Self::push_named_tree_node(
                    nodes,
                    path,
                    ElementKind::TelemetryParameterType(index),
                    Self::parameter_type_label(parameter_type),
                    child_level + 2,
                );
            }
            let parameters = metadata
                .parameter_set
                .as_ref()
                .map(|set| set.content.as_slice())
                .unwrap_or_default();
            Self::push_tree_node(
                nodes,
                path,
                ElementKind::TelemetryParameterSet,
                child_level + 1,
                !parameters.is_empty(),
            );
            for (index, parameter) in parameters.iter().enumerate() {
                let reference = match parameter {
                    xtce::ParameterSetTypeContent::Parameter(parameter) => {
                        Some(parameter.name.clone())
                    }
                    xtce::ParameterSetTypeContent::ParameterRef(_) => None,
                };
                Self::push_named_tree_node(
                    nodes,
                    path,
                    ElementKind::TelemetryParameter(index),
                    Self::parameter_label(parameter),
                    child_level + 2,
                );
                nodes
                    .last_mut()
                    .expect("the parameter tree node was just added")
                    .drag_parameter_reference = reference;
            }
            let containers = metadata
                .container_set
                .as_ref()
                .map(|set| set.content.as_slice())
                .unwrap_or_default();
            Self::push_tree_node(
                nodes,
                path,
                ElementKind::ContainerSet,
                child_level + 1,
                !containers.is_empty(),
            );
            for (index, container) in containers.iter().enumerate() {
                Self::push_named_tree_node(
                    nodes,
                    path,
                    ElementKind::SequenceContainer(index),
                    Self::sequence_container_label(container),
                    child_level + 2,
                );
            }
            let messages = metadata
                .message_set
                .as_ref()
                .map(|set| set.message.as_slice())
                .unwrap_or_default();
            Self::push_tree_node(
                nodes,
                path,
                ElementKind::MessageSet,
                child_level + 1,
                !messages.is_empty(),
            );
            for (index, message) in messages.iter().enumerate() {
                Self::push_named_tree_node(
                    nodes,
                    path,
                    ElementKind::Message(index),
                    message.name.clone(),
                    child_level + 2,
                );
            }
            let streams = metadata
                .stream_set
                .as_ref()
                .map(|set| set.content.as_slice())
                .unwrap_or_default();
            let editable_stream_count = streams
                .iter()
                .filter(|stream| {
                    matches!(
                        stream,
                        xtce::StreamSetTypeContent::FixedFrameStream(_)
                            | xtce::StreamSetTypeContent::VariableFrameStream(_)
                            | xtce::StreamSetTypeContent::CustomStream(_)
                    )
                })
                .count();
            Self::push_tree_node(
                nodes,
                path,
                ElementKind::TelemetryStreamSet,
                child_level + 1,
                editable_stream_count > 0,
            );
            for (index, stream) in streams.iter().enumerate() {
                match stream {
                    xtce::StreamSetTypeContent::FixedFrameStream(stream) => {
                        Self::push_named_tree_node(
                            nodes,
                            path,
                            ElementKind::TelemetryFixedFrameStream(index),
                            stream.name.clone(),
                            child_level + 2,
                        );
                    }
                    xtce::StreamSetTypeContent::VariableFrameStream(stream) => {
                        Self::push_named_tree_node(
                            nodes,
                            path,
                            ElementKind::TelemetryVariableFrameStream(index),
                            stream.name.clone(),
                            child_level + 2,
                        );
                    }
                    xtce::StreamSetTypeContent::CustomStream(stream) => {
                        Self::push_named_tree_node(
                            nodes,
                            path,
                            ElementKind::TelemetryCustomStream(index),
                            stream.name.clone(),
                            child_level + 2,
                        );
                    }
                }
            }
            let algorithms = metadata
                .algorithm_set
                .as_ref()
                .map(|set| set.content.as_slice())
                .unwrap_or_default();
            Self::push_tree_node(
                nodes,
                path,
                ElementKind::TelemetryAlgorithmSet,
                child_level + 1,
                !algorithms.is_empty(),
            );
            for (index, algorithm) in algorithms.iter().enumerate() {
                match algorithm {
                    xtce::AlgorithmSetTypeContent::CustomAlgorithm(algorithm) => {
                        Self::push_named_tree_node(
                            nodes,
                            path,
                            ElementKind::TelemetryCustomAlgorithm(index),
                            algorithm.name.clone(),
                            child_level + 2,
                        );
                    }
                    xtce::AlgorithmSetTypeContent::MathAlgorithm(algorithm) => {
                        Self::push_named_tree_node(
                            nodes,
                            path,
                            ElementKind::TelemetryMathAlgorithm(index),
                            algorithm.name.clone(),
                            child_level + 2,
                        );
                    }
                }
            }
        }
        if let Some(metadata) = &system.command_meta_data {
            Self::push_tree_node(nodes, path, ElementKind::CommandMetaData, child_level, true);
            let parameter_types = metadata
                .parameter_type_set
                .as_ref()
                .map(|set| set.content.as_slice())
                .unwrap_or_default();
            Self::push_tree_node(
                nodes,
                path,
                ElementKind::CommandParameterTypeSet,
                child_level + 1,
                !parameter_types.is_empty(),
            );
            for (index, parameter_type) in parameter_types.iter().enumerate() {
                Self::push_named_tree_node(
                    nodes,
                    path,
                    ElementKind::CommandParameterType(index),
                    Self::parameter_type_label(parameter_type),
                    child_level + 2,
                );
            }
            let parameters = metadata
                .parameter_set
                .as_ref()
                .map(|set| set.content.as_slice())
                .unwrap_or_default();
            Self::push_tree_node(
                nodes,
                path,
                ElementKind::CommandParameterSet,
                child_level + 1,
                !parameters.is_empty(),
            );
            for (index, parameter) in parameters.iter().enumerate() {
                Self::push_named_tree_node(
                    nodes,
                    path,
                    ElementKind::CommandParameter(index),
                    Self::parameter_label(parameter),
                    child_level + 2,
                );
            }
            let argument_types = metadata
                .argument_type_set
                .as_ref()
                .map(|set| set.content.as_slice())
                .unwrap_or_default();
            Self::push_tree_node(
                nodes,
                path,
                ElementKind::ArgumentTypeSet,
                child_level + 1,
                !argument_types.is_empty(),
            );
            for (index, argument_type) in argument_types.iter().enumerate() {
                Self::push_named_tree_node(
                    nodes,
                    path,
                    ElementKind::ArgumentType(index),
                    Self::argument_type_label(argument_type),
                    child_level + 2,
                );
            }
            let meta_commands = metadata
                .meta_command_set
                .as_ref()
                .map(|set| set.content.as_slice())
                .unwrap_or_default();
            Self::push_tree_node(
                nodes,
                path,
                ElementKind::MetaCommandSet,
                child_level + 1,
                !meta_commands.is_empty(),
            );
            for (index, command) in meta_commands.iter().enumerate() {
                Self::push_named_tree_node(
                    nodes,
                    path,
                    ElementKind::MetaCommand(index),
                    Self::meta_command_label(command),
                    child_level + 2,
                );
            }
            let streams = metadata
                .stream_set
                .as_ref()
                .map(|set| set.content.as_slice())
                .unwrap_or_default();
            let editable_stream_count = streams
                .iter()
                .filter(|stream| {
                    matches!(
                        stream,
                        xtce::StreamSetTypeContent::FixedFrameStream(_)
                            | xtce::StreamSetTypeContent::VariableFrameStream(_)
                            | xtce::StreamSetTypeContent::CustomStream(_)
                    )
                })
                .count();
            Self::push_tree_node(
                nodes,
                path,
                ElementKind::CommandStreamSet,
                child_level + 1,
                editable_stream_count > 0,
            );
            for (index, stream) in streams.iter().enumerate() {
                match stream {
                    xtce::StreamSetTypeContent::FixedFrameStream(stream) => {
                        Self::push_named_tree_node(
                            nodes,
                            path,
                            ElementKind::CommandFixedFrameStream(index),
                            stream.name.clone(),
                            child_level + 2,
                        );
                    }
                    xtce::StreamSetTypeContent::VariableFrameStream(stream) => {
                        Self::push_named_tree_node(
                            nodes,
                            path,
                            ElementKind::CommandVariableFrameStream(index),
                            stream.name.clone(),
                            child_level + 2,
                        );
                    }
                    xtce::StreamSetTypeContent::CustomStream(stream) => {
                        Self::push_named_tree_node(
                            nodes,
                            path,
                            ElementKind::CommandCustomStream(index),
                            stream.name.clone(),
                            child_level + 2,
                        );
                    }
                }
            }
            let command_containers = metadata
                .command_container_set
                .as_ref()
                .map(|set| set.command_container.as_slice())
                .unwrap_or_default();
            Self::push_tree_node(
                nodes,
                path,
                ElementKind::CommandContainerSet,
                child_level + 1,
                !command_containers.is_empty(),
            );
            for (index, container) in command_containers.iter().enumerate() {
                Self::push_named_tree_node(
                    nodes,
                    path,
                    ElementKind::CommandContainer(index),
                    container.name.clone(),
                    child_level + 2,
                );
            }
            let algorithms = metadata
                .algorithm_set
                .as_ref()
                .map(|set| set.content.as_slice())
                .unwrap_or_default();
            Self::push_tree_node(
                nodes,
                path,
                ElementKind::CommandAlgorithmSet,
                child_level + 1,
                !algorithms.is_empty(),
            );
            for (index, algorithm) in algorithms.iter().enumerate() {
                match algorithm {
                    xtce::AlgorithmSetTypeContent::CustomAlgorithm(algorithm) => {
                        Self::push_named_tree_node(
                            nodes,
                            path,
                            ElementKind::CommandCustomAlgorithm(index),
                            algorithm.name.clone(),
                            child_level + 2,
                        );
                    }
                    xtce::AlgorithmSetTypeContent::MathAlgorithm(algorithm) => {
                        Self::push_named_tree_node(
                            nodes,
                            path,
                            ElementKind::CommandMathAlgorithm(index),
                            algorithm.name.clone(),
                            child_level + 2,
                        );
                    }
                }
            }
        }
        if let Some(set) = system.service_set.as_ref() {
            Self::push_tree_node(
                nodes,
                path,
                ElementKind::ServiceSet,
                child_level,
                !set.service.is_empty(),
            );
            for (index, service) in set.service.iter().enumerate() {
                Self::push_named_tree_node(
                    nodes,
                    path,
                    ElementKind::Service(index),
                    service.name.clone(),
                    child_level + 1,
                );
            }
        }

        for (index, child) in system.space_system.iter().enumerate() {
            path.push(index);
            Self::collect_tree_nodes(child, path, level + 1, nodes);
            path.pop();
        }
    }

    fn push_tree_node(
        nodes: &mut Vec<TreeNode>,
        system_path: &[usize],
        kind: ElementKind,
        level: usize,
        has_children: bool,
    ) {
        nodes.push(TreeNode {
            selection: ElementSelection {
                system_path: system_path.to_vec(),
                kind,
            },
            label: kind.label().to_owned(),
            drag_parameter_reference: None,
            level,
            has_children,
            can_add_telemetry_metadata: false,
            can_add_command_metadata: false,
            can_add_service_set: false,
        });
    }

    fn push_named_tree_node(
        nodes: &mut Vec<TreeNode>,
        system_path: &[usize],
        kind: ElementKind,
        label: String,
        level: usize,
    ) {
        nodes.push(TreeNode {
            selection: ElementSelection {
                system_path: system_path.to_vec(),
                kind,
            },
            label,
            drag_parameter_reference: None,
            level,
            has_children: false,
            can_add_telemetry_metadata: false,
            can_add_command_metadata: false,
            can_add_service_set: false,
        });
    }

    fn parameter_label(parameter: &xtce::ParameterSetTypeContent) -> String {
        match parameter {
            xtce::ParameterSetTypeContent::Parameter(parameter) => parameter.name.clone(),
            xtce::ParameterSetTypeContent::ParameterRef(parameter) => {
                format!("→ {}", parameter.parameter_ref)
            }
        }
    }

    fn sequence_container_label(container: &xtce::ContainerSetTypeContent) -> String {
        match container {
            xtce::ContainerSetTypeContent::SequenceContainer(container) => container.name.clone(),
        }
    }

    fn parameter_type_label(parameter_type: &xtce::ParameterTypeSetTypeContent) -> String {
        match parameter_type {
            xtce::ParameterTypeSetTypeContent::StringParameterType(value) => value.name.clone(),
            xtce::ParameterTypeSetTypeContent::EnumeratedParameterType(value) => value.name.clone(),
            xtce::ParameterTypeSetTypeContent::IntegerParameterType(value) => value.name.clone(),
            xtce::ParameterTypeSetTypeContent::BinaryParameterType(value) => value.name.clone(),
            xtce::ParameterTypeSetTypeContent::FloatParameterType(value) => value.name.clone(),
            xtce::ParameterTypeSetTypeContent::BooleanParameterType(value) => value.name.clone(),
            xtce::ParameterTypeSetTypeContent::RelativeTimeParameterType(value) => {
                value.name.clone()
            }
            xtce::ParameterTypeSetTypeContent::AbsoluteTimeParameterType(value) => {
                value.name.clone()
            }
            xtce::ParameterTypeSetTypeContent::ArrayParameterType(value) => value.name.clone(),
            xtce::ParameterTypeSetTypeContent::AggregateParameterType(value) => value.name.clone(),
        }
    }

    fn argument_type_label(argument_type: &xtce::ArgumentTypeSetTypeContent) -> String {
        match argument_type {
            xtce::ArgumentTypeSetTypeContent::StringArgumentType(value) => value.name.clone(),
            xtce::ArgumentTypeSetTypeContent::EnumeratedArgumentType(value) => value.name.clone(),
            xtce::ArgumentTypeSetTypeContent::IntegerArgumentType(value) => value.name.clone(),
            xtce::ArgumentTypeSetTypeContent::BinaryArgumentType(value) => value.name.clone(),
            xtce::ArgumentTypeSetTypeContent::FloatArgumentType(value) => value.name.clone(),
            xtce::ArgumentTypeSetTypeContent::BooleanArgumentType(value) => value.name.clone(),
            xtce::ArgumentTypeSetTypeContent::RelativeTimeArgumentType(value) => value.name.clone(),
            xtce::ArgumentTypeSetTypeContent::AbsoluteTimeArgumentType(value) => value.name.clone(),
            xtce::ArgumentTypeSetTypeContent::ArrayArgumentType(value) => value.name.clone(),
            xtce::ArgumentTypeSetTypeContent::AggregateArgumentType(value) => value.name.clone(),
        }
    }

    fn meta_command_label(command: &xtce::MetaCommandSetTypeContent) -> String {
        match command {
            xtce::MetaCommandSetTypeContent::MetaCommand(value) => value.name.clone(),
            xtce::MetaCommandSetTypeContent::MetaCommandRef(value) => format!("→ {value}"),
            xtce::MetaCommandSetTypeContent::BlockMetaCommand(value) => value.name.clone(),
        }
    }

    fn algorithm_label(algorithm: &xtce::AlgorithmSetTypeContent) -> String {
        match algorithm {
            xtce::AlgorithmSetTypeContent::CustomAlgorithm(value) => value.name.clone(),
            xtce::AlgorithmSetTypeContent::MathAlgorithm(value) => value.name.clone(),
        }
    }

    fn stream_label(stream: &xtce::StreamSetTypeContent) -> String {
        match stream {
            xtce::StreamSetTypeContent::FixedFrameStream(value) => value.name.clone(),
            xtce::StreamSetTypeContent::VariableFrameStream(value) => value.name.clone(),
            xtce::StreamSetTypeContent::CustomStream(value) => value.name.clone(),
        }
    }

    fn system_type_label(system_type: &xtce::SystemTypeType) -> &'static str {
        match system_type {
            xtce::SystemTypeType::Asset => "asset",
            xtce::SystemTypeType::AssetGroup => "asset group",
            xtce::SystemTypeType::AssetComponent => "asset component",
            xtce::SystemTypeType::Unknown => "unknown",
        }
    }
}

impl ElementTree {
    fn new(
        root: &xtce::SpaceSystem,
        selection: ElementSelection,
        file_name: String,
        editor: WeakEntity<XtceEditor>,
        window: &mut Window,
        cx: &mut Context<XtceEditor>,
    ) -> Entity<Self> {
        let mut nodes = Vec::new();
        XtceDocument::collect_tree_nodes(root, &mut Vec::new(), 0, &mut nodes);
        cx.new(move |cx| {
            let search_input =
                cx.new(|cx| InputState::new(window, cx).placeholder("Filter elements…"));
            let tree_state = cx.new(|cx| TreeState::new(cx));
            let search_subscription = cx.subscribe(
                &search_input,
                |this: &mut ElementTree, _, event: &ComponentInputEvent, cx| {
                    if matches!(event, ComponentInputEvent::Change) {
                        this.filter_collapsed.clear();
                        this.rebuild(cx);
                    }
                },
            );
            let tree_subscription = cx.subscribe(
                &tree_state,
                |this: &mut ElementTree, _, event: &TreeEvent, _| {
                    let id = match event {
                        TreeEvent::Expanded(id) | TreeEvent::Collapsed(id) => id,
                    };
                    let Some(node) = this.nodes.get(id) else {
                        return;
                    };
                    match event {
                        TreeEvent::Expanded(_) => {
                            this.collapsed.remove(&node.selection);
                            this.filter_collapsed.remove(&node.selection);
                        }
                        TreeEvent::Collapsed(_) => {
                            this.collapsed.insert(node.selection.clone());
                            this.filter_collapsed.insert(node.selection.clone());
                        }
                    }
                },
            );
            let mut this = Self {
                search_input,
                collapsed: Self::collapsed_by_default(root),
                filter_collapsed: HashSet::new(),
                tree_state,
                nodes: HashMap::new(),
                ordered_nodes: Vec::new(),
                node_count: 0,
                filter_match_count: None,
                selection_outside_filter: false,
                hidden_selection_ancestor: None,
                file_name,
                selection: selection.clone(),
                editor,
                _subscriptions: vec![search_subscription, tree_subscription],
            };
            this.load_nodes(nodes, selection, this.file_name.clone(), cx);
            this
        })
    }

    fn node_id(selection: &ElementSelection) -> SharedString {
        let path = if selection.system_path.is_empty() {
            "root".to_owned()
        } else {
            selection
                .system_path
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join("-")
        };
        format!("{path}:{:?}", selection.kind).into()
    }

    fn expand(&mut self, selection: &ElementSelection) {
        self.collapsed.remove(selection);
        self.filter_collapsed.remove(selection);
    }

    fn load_nodes(
        &mut self,
        nodes: Vec<TreeNode>,
        selection: ElementSelection,
        file_name: String,
        cx: &mut Context<Self>,
    ) {
        self.node_count = nodes.len();
        self.file_name = file_name;
        self.selection = selection.clone();
        self.ordered_nodes = nodes;
        self.nodes = self
            .ordered_nodes
            .iter()
            .cloned()
            .map(|node| (Self::node_id(&node.selection), node))
            .collect();
        self.rebuild_with_selection(Some(&selection), cx);
    }

    fn rebuild(&mut self, cx: &mut Context<Self>) {
        let selection = self.selection.clone();
        self.rebuild_with_selection(Some(&selection), cx);
    }

    fn rebuild_with_selection(
        &mut self,
        selected: Option<&ElementSelection>,
        cx: &mut Context<Self>,
    ) {
        let query = self
            .search_input
            .read(cx)
            .value()
            .trim()
            .to_ascii_lowercase();
        self.filter_match_count = (!query.is_empty()).then(|| {
            self.ordered_nodes
                .iter()
                .filter(|node| node.label.to_ascii_lowercase().contains(&query))
                .count()
        });
        self.selection_outside_filter = !query.is_empty()
            && selected.is_some_and(|selection| {
                !Self::selection_is_filter_context(&self.ordered_nodes, selection, &query)
            });
        let mut index = 0;
        let items = Self::build_items(
            &self.ordered_nodes,
            &mut index,
            0,
            &query,
            &self.collapsed,
            &self.filter_collapsed,
            selected,
        );
        let selected_id = selected.map(Self::node_id);
        self.hidden_selection_ancestor = selected_id
            .as_ref()
            .and_then(|id| Self::find_hidden_selection_ancestor(&items, id))
            .map(|item| item.id.clone());
        let selected_item = selected_id
            .as_ref()
            .and_then(|id| Self::find_visible_item(&items, id))
            .cloned();
        self.tree_state.update(cx, |state, cx| {
            state.set_items(items, cx);
            state.set_selected_item(selected_item.as_ref(), cx);
        });
        cx.notify();
    }

    fn build_items(
        nodes: &[TreeNode],
        index: &mut usize,
        level: usize,
        query: &str,
        collapsed: &HashSet<ElementSelection>,
        filter_collapsed: &HashSet<ElementSelection>,
        selected: Option<&ElementSelection>,
    ) -> Vec<TreeItem> {
        let mut items = Vec::new();
        while *index < nodes.len() {
            if nodes[*index].level < level {
                break;
            }
            if nodes[*index].level > level {
                break;
            }
            let node = nodes[*index].clone();
            *index += 1;
            let child_level = nodes
                .get(*index)
                .filter(|next| next.level > level)
                .map(|next| next.level);
            let children = child_level
                .map(|level| {
                    Self::build_items(
                        nodes,
                        index,
                        level,
                        query,
                        collapsed,
                        filter_collapsed,
                        selected,
                    )
                })
                .unwrap_or_default();
            let matches = query.is_empty() || node.label.to_ascii_lowercase().contains(query);
            let selected = selected.is_some_and(|selection| node.selection == *selection);
            if !matches && !selected && children.is_empty() {
                continue;
            }
            let expanded = if query.is_empty() {
                !collapsed.contains(&node.selection)
            } else {
                !children.is_empty() && !filter_collapsed.contains(&node.selection)
            };
            items.push(
                TreeItem::new(Self::node_id(&node.selection), node.label)
                    .expanded(expanded)
                    .children(children),
            );
        }
        items
    }

    fn selection_is_filter_context(
        nodes: &[TreeNode],
        selection: &ElementSelection,
        query: &str,
    ) -> bool {
        let Some(index) = nodes.iter().position(|node| node.selection == *selection) else {
            return false;
        };
        let selected_level = nodes[index].level;
        nodes[index..]
            .iter()
            .take_while(|node| node.selection == *selection || node.level > selected_level)
            .any(|node| node.label.to_ascii_lowercase().contains(query))
    }

    fn find_visible_item<'a>(items: &'a [TreeItem], id: &SharedString) -> Option<&'a TreeItem> {
        items.iter().find_map(|item| {
            if &item.id == id {
                Some(item)
            } else if item.is_expanded() {
                Self::find_visible_item(&item.children, id)
            } else {
                None
            }
        })
    }

    fn find_hidden_selection_ancestor<'a>(
        items: &'a [TreeItem],
        id: &SharedString,
    ) -> Option<&'a TreeItem> {
        fn contains(items: &[TreeItem], id: &SharedString) -> bool {
            items
                .iter()
                .any(|item| &item.id == id || contains(&item.children, id))
        }

        items.iter().find_map(|item| {
            if &item.id == id {
                None
            } else if contains(&item.children, id) {
                if item.is_expanded() {
                    Self::find_hidden_selection_ancestor(&item.children, id)
                } else {
                    Some(item)
                }
            } else {
                None
            }
        })
    }

    fn render_entry(
        &mut self,
        index: usize,
        entry: &gpui_component::tree::TreeEntry,
        selected: bool,
        cx: &mut Context<Self>,
    ) -> ListItem {
        let item = entry.item();
        let Some(node) = self.nodes.get(&item.id).cloned() else {
            return ListItem::new(index).child(item.label.clone());
        };
        let contains_hidden_selection = self
            .hidden_selection_ancestor
            .as_ref()
            .is_some_and(|id| id == &item.id);
        let editor = self.editor.clone();
        let selection = node.selection.clone();
        let add_parent = node.selection.clone();
        let stream_parent = node.selection.clone();
        let algorithm_parent = node.selection.clone();
        let metadata_parent = node.selection.clone();
        let delete_selection = node.selection.clone();
        let delete_editor = editor.clone();
        let can_add_child = node.selection.kind.can_add_child();
        let can_delete = node.selection.kind.can_delete();
        let can_add_stream = matches!(
            node.selection.kind,
            ElementKind::TelemetryStreamSet | ElementKind::CommandStreamSet
        );
        let can_add_algorithm = matches!(
            node.selection.kind,
            ElementKind::TelemetryAlgorithmSet | ElementKind::CommandAlgorithmSet
        );
        let can_add_element = node.selection.kind == ElementKind::SpaceSystem;
        let directory_only = node.selection.kind.is_directory_only();
        let dragged_parameter = node
            .drag_parameter_reference
            .clone()
            .map(|reference| DraggedTelemetryParameter { reference });
        let icon = node.selection.kind.tree_icon(entry.is_expanded());
        let row_group: SharedString = format!("tree-row-{index}").into();
        ListItem::new(index)
            .w_full()
            .rounded_md()
            .px_2()
            .pl(px(10. + entry.depth() as f32 * 18.))
            .selected(selected)
            .child(
                h_flex()
                    .id(format!("tree-node-drag-{index}"))
                    .group(row_group.clone())
                    .w_full()
                    .gap_2()
                    .when(contains_hidden_selection, |row| {
                        row.rounded_sm()
                            .border_l_2()
                            .border_color(cx.theme().primary)
                            .bg(cx.theme().sidebar_accent.opacity(0.5))
                            .tooltip(|window, cx| {
                                Tooltip::new("Contains the current selection").build(window, cx)
                            })
                    })
                    .when_some(dragged_parameter, |row, dragged| {
                        row.cursor_grab().on_drag(dragged, |dragged, _, _, cx| {
                            cx.stop_propagation();
                            let reference = dragged.reference.clone();
                            cx.new(|_| ParameterDragPreview { reference })
                        })
                    })
                    .child(Icon::new(icon).small())
                    .child(div().flex_1().truncate().child(item.label.clone()))
                    .when(
                        can_add_child && !can_add_stream && !can_add_algorithm,
                        |row| {
                            let editor = editor.clone();
                            row.child(
                                Button::new(format!("{}-add", item.id))
                                    .ghost()
                                    .xsmall()
                                    .icon(IconName::Plus)
                                    .on_click(move |_, window, cx| {
                                        cx.stop_propagation();
                                        _ = editor.update(cx, |this, cx| {
                                            this.add_tree_child(&add_parent, window, cx);
                                        });
                                    }),
                            )
                        },
                    )
                    .when(can_add_stream, |row| {
                        let stream_editor = editor.clone();
                        row.child(
                            Button::new(format!("{}-add-stream", item.id))
                                .ghost()
                                .xsmall()
                                .icon(IconName::Plus)
                                .dropdown_menu(move |menu, _, _| {
                                    let fixed_editor = stream_editor.clone();
                                    let fixed_parent = stream_parent.clone();
                                    let variable_editor = stream_editor.clone();
                                    let variable_parent = stream_parent.clone();
                                    let custom_editor = stream_editor.clone();
                                    let custom_parent = stream_parent.clone();
                                    menu.item(PopupMenuItem::new("FixedFrameStream").on_click(
                                        move |_, window, cx| {
                                            _ = fixed_editor.update(cx, |this, cx| {
                                                this.add_stream_child(
                                                    &fixed_parent,
                                                    StreamChildKind::Fixed,
                                                    window,
                                                    cx,
                                                );
                                            });
                                        },
                                    ))
                                    .item(PopupMenuItem::new("VariableFrameStream").on_click(
                                        move |_, window, cx| {
                                            _ = variable_editor.update(cx, |this, cx| {
                                                this.add_stream_child(
                                                    &variable_parent,
                                                    StreamChildKind::Variable,
                                                    window,
                                                    cx,
                                                );
                                            });
                                        },
                                    ))
                                    .item(
                                        PopupMenuItem::new("CustomStream").on_click(
                                            move |_, window, cx| {
                                                _ = custom_editor.update(cx, |this, cx| {
                                                    this.add_stream_child(
                                                        &custom_parent,
                                                        StreamChildKind::Custom,
                                                        window,
                                                        cx,
                                                    );
                                                });
                                            },
                                        ),
                                    )
                                }),
                        )
                    })
                    .when(can_add_algorithm, |row| {
                        let algorithm_editor = editor.clone();
                        row.child(
                            Button::new(format!("{}-add-algorithm", item.id))
                                .ghost()
                                .xsmall()
                                .icon(IconName::Plus)
                                .dropdown_menu(move |menu, _, _| {
                                    let custom_editor = algorithm_editor.clone();
                                    let custom_parent = algorithm_parent.clone();
                                    let math_editor = algorithm_editor.clone();
                                    let math_parent = algorithm_parent.clone();
                                    menu.item(PopupMenuItem::new("CustomAlgorithm").on_click(
                                        move |_, window, cx| {
                                            _ = custom_editor.update(cx, |this, cx| {
                                                this.add_algorithm_child(
                                                    &custom_parent,
                                                    AlgorithmChildKind::Custom,
                                                    window,
                                                    cx,
                                                );
                                            });
                                        },
                                    ))
                                    .item(
                                        PopupMenuItem::new("MathAlgorithm").on_click(
                                            move |_, window, cx| {
                                                _ = math_editor.update(cx, |this, cx| {
                                                    this.add_algorithm_child(
                                                        &math_parent,
                                                        AlgorithmChildKind::Math,
                                                        window,
                                                        cx,
                                                    );
                                                });
                                            },
                                        ),
                                    )
                                }),
                        )
                    })
                    .when(can_add_element, |row| {
                        row.child(
                            Button::new(format!("{}-add-metadata", item.id))
                                .ghost()
                                .xsmall()
                                .icon(IconName::Plus)
                                .dropdown_menu(move |menu, _, _| {
                                    let space_system_editor = editor.clone();
                                    let space_system_parent = metadata_parent.clone();
                                    let telemetry_editor = editor.clone();
                                    let telemetry_parent = metadata_parent.clone();
                                    let command_editor = editor.clone();
                                    let command_parent = metadata_parent.clone();
                                    let service_editor = editor.clone();
                                    let service_parent = metadata_parent.clone();
                                    menu.item(
                                        PopupMenuItem::new("SpaceSystem")
                                            .icon(IconName::Globe)
                                            .on_click(move |_, window, cx| {
                                                _ = space_system_editor.update(cx, |this, cx| {
                                                    this.add_space_system(
                                                        &space_system_parent,
                                                        window,
                                                        cx,
                                                    );
                                                });
                                            }),
                                    )
                                    .separator()
                                    .item(
                                        PopupMenuItem::new("TelemetryMetaData")
                                            .icon(IconName::ChartPie)
                                            .disabled(!node.can_add_telemetry_metadata)
                                            .on_click(move |_, window, cx| {
                                                _ = telemetry_editor.update(cx, |this, cx| {
                                                    this.add_metadata(
                                                        &telemetry_parent,
                                                        ElementKind::TelemetryMetaData,
                                                        window,
                                                        cx,
                                                    );
                                                });
                                            }),
                                    )
                                    .item(
                                        PopupMenuItem::new("CommandMetaData")
                                            .icon(IconName::SquareTerminal)
                                            .disabled(!node.can_add_command_metadata)
                                            .on_click(move |_, window, cx| {
                                                _ = command_editor.update(cx, |this, cx| {
                                                    this.add_metadata(
                                                        &command_parent,
                                                        ElementKind::CommandMetaData,
                                                        window,
                                                        cx,
                                                    );
                                                });
                                            }),
                                    )
                                    .item(
                                        PopupMenuItem::new("ServiceSet")
                                            .icon(IconName::Building2)
                                            .disabled(!node.can_add_service_set)
                                            .on_click(move |_, window, cx| {
                                                _ = service_editor.update(cx, |this, cx| {
                                                    this.add_metadata(
                                                        &service_parent,
                                                        ElementKind::ServiceSet,
                                                        window,
                                                        cx,
                                                    );
                                                });
                                            }),
                                    )
                                }),
                        )
                    })
                    .when(can_delete, |row| {
                        row.child(
                            Button::new(format!("{}-actions", item.id))
                                .ghost()
                                .xsmall()
                                .icon(IconName::Ellipsis)
                                .when(!selected, |button| {
                                    button
                                        .invisible()
                                        .group_hover(row_group.clone(), |button| button.visible())
                                })
                                .dropdown_menu(move |menu, _, _| {
                                    let editor = delete_editor.clone();
                                    let selection = delete_selection.clone();
                                    menu.item(PopupMenuItem::new("Delete…").on_click(
                                        move |_, window, cx| {
                                            _ = editor.update(cx, |editor, cx| {
                                                editor.request_delete(&selection, window, cx);
                                            });
                                        },
                                    ))
                                }),
                        )
                    }),
            )
            .when(!directory_only, |item| {
                item.on_click(cx.listener(move |this, _, window, _| {
                    let editor = this.editor.clone();
                    let selection = selection.clone();
                    window.on_next_frame(move |window, cx| {
                        _ = editor.update(cx, |editor, cx| {
                            if editor.document.selection == selection {
                                return;
                            }
                            editor.save_selected_element(cx);
                            editor.document.selection = selection;
                            editor.load_selected_element(window, cx);
                        });
                    });
                }))
            })
    }

    fn collapsed_by_default(root: &xtce::SpaceSystem) -> HashSet<ElementSelection> {
        let mut nodes = Vec::new();
        XtceDocument::collect_tree_nodes(root, &mut Vec::new(), 0, &mut nodes);
        nodes
            .into_iter()
            .filter(|node| node.has_children)
            .map(|node| node.selection)
            .collect()
    }

    #[cfg(test)]
    fn visible_nodes(nodes: Vec<TreeNode>, collapsed: &HashSet<ElementSelection>) -> Vec<TreeNode> {
        let mut hidden_below_level = None;
        let mut visible = Vec::with_capacity(nodes.len());

        for node in nodes {
            if let Some(level) = hidden_below_level {
                if node.level > level {
                    continue;
                }
                hidden_below_level = None;
            }
            if node.has_children && collapsed.contains(&node.selection) {
                hidden_below_level = Some(node.level);
            }
            visible.push(node);
        }

        visible
    }
}

impl ElementInspector {
    fn section(
        title: &'static str,
        description: &'static str,
        content: impl IntoElement,
        cx: &App,
    ) -> Div {
        v_flex()
            .w_full()
            .gap_5()
            .p_5()
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .child(
                v_flex()
                    .gap_1()
                    .child(div().text_base().font_semibold().child(title))
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(description),
                    ),
            )
            .child(content)
    }
}

impl Render for ElementTree {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        let count_label = self.filter_match_count.map_or_else(
            || format!("{} elements", self.node_count),
            |count| format!("{count} of {} elements", self.node_count),
        );
        let filter_notice = match (self.filter_match_count, self.selection_outside_filter) {
            (Some(0), true) => {
                Some("No matching elements. The current selection is shown for context.")
            }
            (Some(0), false) => Some("No matching elements."),
            (Some(_), true) => Some("The current selection does not match this filter."),
            _ => None,
        };
        v_flex()
            .w_full()
            .h_full()
            .border_r_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().sidebar)
            .child(
                h_flex()
                    .h(px(58.))
                    .px_4()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(self.file_name.clone()),
                    ),
            )
            .child(
                div()
                    .px_3()
                    .py_3()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(
                        Input::new(&self.search_input)
                            .prefix(IconName::Search)
                            .cleanable(true),
                    ),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_h_0()
                    .when_some(filter_notice, |tree_panel, notice| {
                        tree_panel.child(
                            div()
                                .px_4()
                                .py_2()
                                .flex_none()
                                .border_b_1()
                                .border_color(cx.theme().border)
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(notice),
                        )
                    })
                    .child(
                        div()
                            .id("element-tree")
                            .flex_1()
                            .min_h_0()
                            .p_2()
                            .child(tree(
                                &self.tree_state,
                                move |index, entry, selected, _, cx| {
                                    view.update(cx, |this, cx| {
                                        this.render_entry(index, entry, selected, cx)
                                    })
                                },
                            )),
                    ),
            )
            .child(
                h_flex()
                    .h(px(38.))
                    .px_4()
                    .flex_none()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(count_label)
                    .child(div().flex_1())
                    .child("XTCE 1.3"),
            )
    }
}

impl ElementInspector {
    fn render_breadcrumb(
        &self,
        document: &XtceDocument,
        current_label: String,
        cx: &mut Context<XtceEditor>,
    ) -> Div {
        let items = breadcrumb_items(&document.root, &document.selection, current_label);
        let item_count = items.len();
        let mut breadcrumb = h_flex().min_w_0().gap_1().text_sm().overflow_hidden();

        for (index, item) in items.into_iter().enumerate() {
            if index > 0 {
                breadcrumb = breadcrumb.child(
                    Icon::new(IconName::ChevronRight)
                        .xsmall()
                        .flex_none()
                        .text_color(cx.theme().muted_foreground),
                );
            }

            let is_current = index + 1 == item_count;
            let full_label = item.label;
            let visible_label = abbreviated_breadcrumb_label(&full_label);
            if let Some(system_path) = item.system_path.filter(|_| !is_current) {
                let tooltip = full_label.clone();
                breadcrumb = breadcrumb.child(
                    Button::new(format!("breadcrumb-space-system-{index}"))
                        .xsmall()
                        .compact()
                        .link()
                        .label(visible_label)
                        .tooltip(tooltip)
                        .on_click(cx.listener(move |this, _, window, cx| {
                            let selection = ElementSelection {
                                system_path: system_path.clone(),
                                kind: ElementKind::SpaceSystem,
                            };
                            if this.document.selection == selection {
                                return;
                            }
                            this.save_selected_element(cx);
                            this.document.selection = selection;
                            this.load_selected_element(window, cx);
                        })),
                );
            } else {
                let tooltip = full_label.clone();
                breadcrumb = breadcrumb.child(
                    div()
                        .id(format!("breadcrumb-item-{index}"))
                        .min_w_0()
                        .max_w(px(220.))
                        .truncate()
                        .when(is_current, |item| item.font_medium())
                        .when(!is_current, |item| {
                            item.text_color(cx.theme().muted_foreground)
                        })
                        .tooltip(move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx))
                        .child(visible_label),
                );
            }
        }

        breadcrumb
    }

    fn render_structural_element(
        &self,
        document: &XtceDocument,
        cx: &mut Context<XtceEditor>,
    ) -> Div {
        let owner_name = document.selected_system().name.clone();
        let kind = document.selection.kind;
        let element_name = self
            .forms
            .element_title(kind, document.selected_system(), cx);
        let name_editor = self
            .forms
            .render_name_editor(kind, document.selected_system(), cx);
        let form = self.forms.render(kind, document.selected_system(), cx);
        v_flex()
            .flex_1()
            .min_w_0()
            .h_full()
            .bg(cx.theme().muted.opacity(0.28))
            .child(
                h_flex()
                    .h(px(58.))
                    .px_6()
                    .flex_none()
                    .justify_between()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().background)
                    .child(self.render_breadcrumb(document, element_name.clone(), cx)),
            )
            .child(
                v_flex()
                    .id("structural-editor-scroll")
                    .flex_1()
                    .overflow_y_scroll()
                    .items_center()
                    .child(
                        v_flex()
                            .w_full()
                            .max_w(px(920.))
                            .p_7()
                            .gap_6()
                            .child(
                                v_flex()
                                    .gap_2()
                                    .child(if let Some(name_editor) = name_editor {
                                        name_editor
                                    } else {
                                        div()
                                            .text_2xl()
                                            .font_semibold()
                                            .child(element_name.clone())
                                    })
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(format!(
                                                "This element belongs to the {owner_name} space system."
                                            )),
                                    ),
                            )
                            .child(
                                v_flex()
                                    .w_full()
                                    .p_5()
                                    .rounded_lg()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .bg(cx.theme().background)
                                    .child(form),
                            ),
                    ),
            )
    }

    fn render(&self, document: &XtceDocument, cx: &mut Context<XtceEditor>) -> Div {
        if document.selection.kind != ElementKind::SpaceSystem {
            return self.render_structural_element(document, cx);
        }

        let system_type = {
            let system = document.selected_system();
            XtceDocument::system_type_label(&system.system_type)
        };
        let selected_name =
            self.forms
                .element_title(ElementKind::SpaceSystem, document.selected_system(), cx);
        let name_editor = self
            .forms
            .render_name_editor(ElementKind::SpaceSystem, document.selected_system(), cx)
            .expect("SpaceSystem has a name editor");

        let identity_fields =
            self.forms
                .render(ElementKind::SpaceSystem, document.selected_system(), cx);
        let description_fields = self.forms.render_space_system_description(cx);
        let alias_fields = self.forms.render_space_system_aliases(cx);
        let ancillary_data_fields = self.forms.render_space_system_ancillary_data(cx);
        let header_fields = self.forms.render_space_system_header(cx);

        v_flex()
            .flex_1()
            .min_w_0()
            .h_full()
            .bg(cx.theme().muted.opacity(0.28))
            .child(
                h_flex()
                    .h(px(58.))
                    .px_6()
                    .flex_none()
                    .justify_between()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().background)
                    .child(self.render_breadcrumb(document, selected_name, cx)),
            )
            .child(
                v_flex()
                    .id("editor-scroll")
                    .flex_1()
                    .overflow_y_scroll()
                    .items_center()
                    .child(
                        v_flex()
                            .w_full()
                            .max_w(px(920.))
                            .p_7()
                            .gap_6()
                            .child(
                                h_flex()
                                    .items_start()
                                    .justify_between()
                                    .child(
                                        v_flex()
                                            .gap_2()
                                            .child(name_editor)
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child(
                                                        "Review and edit the selected XTCE element.",
                                                    ),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .px_2()
                                            .py_1()
                                            .rounded_md()
                                            .bg(cx.theme().secondary)
                                            .text_xs()
                                            .text_color(cx.theme().secondary_foreground)
                                            .child(system_type),
                                    ),
                            )
                            .child(Self::section(
                                "Identity",
                                "Core attributes used to identify this space system.",
                                identity_fields,
                                cx,
                            ))
                            .child(Self::section(
                                "Description",
                                "Human-readable context for operators and maintainers.",
                                description_fields,
                                cx,
                            ))
                            .child(Self::section(
                                "AliasSet",
                                "Alternative names used by external systems and operators.",
                                alias_fields,
                                cx,
                            ))
                            .child(Self::section(
                                "AncillaryDataSet",
                                "Additional metadata and related resources.",
                                ancillary_data_fields,
                                cx,
                            ))
                            .child(Self::section(
                                "Header",
                                "Document version, classification, validation, and history.",
                                header_fields,
                                cx,
                            ))
                            .child(
                                h_flex()
                                    .p_4()
                                    .rounded_lg()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .bg(cx.theme().background)
                                    .gap_3()
                                    .child(
                                        Icon::new(IconName::Info)
                                            .small()
                                            .text_color(cx.theme().primary),
                                    )
                                    .child(
                                        v_flex()
                                            .gap_1()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_medium()
                                                    .child("Schema defaults are applied"),
                                            )
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child(
                                                        "systemType and assetType use “unknown” when omitted.",
                                                    ),
                                            ),
                                    ),
                            ),
                    ),
            )
    }
}

impl Render for XtceEditor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sheet_layer = Root::render_sheet_layer(window, cx);
        let dialog_layer = Root::render_dialog_layer(window, cx);
        let notification_layer = Root::render_notification_layer(window, cx);

        v_flex()
            .on_action(cx.listener(|this, _: &NewDocument, window, cx| {
                this.new_document(window, cx);
            }))
            .on_action(cx.listener(|this, _: &OpenDocument, window, cx| {
                this.open_document(window, cx);
            }))
            .on_action(cx.listener(|this, _: &SaveDocument, window, cx| {
                this.save_document(window, cx);
            }))
            .on_action(cx.listener(|this, _: &ExportApplicationLog, window, cx| {
                this.export_application_log(window, cx);
            }))
            .relative()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(
                TitleBar::new().child(
                    h_flex()
                        .size_full()
                        .child(self.chrome.app_menu_bar.clone())
                        .child(div().flex_1())
                        .child(
                            div()
                                .pr_3()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("XTCE Editor"),
                        ),
                ),
            )
            .child(
                div().flex_1().min_h_0().overflow_hidden().child(
                    h_resizable("workspace")
                        .child(
                            resizable_panel()
                                .size(px(292.))
                                .size_range(px(220.)..px(520.))
                                .child(self.tree.clone()),
                        )
                        .child(resizable_panel().child(self.inspector.render(&self.document, cx))),
                ),
            )
            .children(sheet_layer)
            .children(dialog_layer)
            .children(notification_layer)
    }
}

fn build_menus() -> Vec<Menu> {
    vec![
        Menu {
            name: "File".into(),
            items: vec![
                MenuItem::action("New File", NewDocument),
                MenuItem::separator(),
                MenuItem::action("Open…", OpenDocument),
                MenuItem::action("Save File…", SaveDocument),
            ],
            disabled: false,
        },
        Menu {
            name: "Edit".into(),
            items: vec![
                MenuItem::action("Undo", gpui_component::input::Undo),
                MenuItem::action("Redo", gpui_component::input::Redo),
                MenuItem::separator(),
                MenuItem::action("Cut", gpui_component::input::Cut),
                MenuItem::action("Copy", gpui_component::input::Copy),
                MenuItem::action("Paste", gpui_component::input::Paste),
            ],
            disabled: false,
        },
        Menu {
            name: "Help".into(),
            items: vec![MenuItem::action(
                "Save Application Log…",
                ExportApplicationLog,
            )],
            disabled: false,
        },
    ]
}

fn startup_document(path: Option<&std::path::Path>) -> Result<XtceDocument, String> {
    path.map_or_else(|| Ok(XtceDocument::untitled()), XtceDocument::read)
}

fn main() {
    support_log::init();
    let startup_path = std::env::args_os().nth(1).map(std::path::PathBuf::from);
    if let Some(path) = &startup_path {
        support_log::event(
            "INFO",
            &format!("opening startup document {}", path.display()),
        );
    } else {
        support_log::event("INFO", "starting with an empty document");
    }
    let startup_result = startup_document(startup_path.as_deref());
    if let Err(error) = &startup_result {
        support_log::event("ERROR", error);
    }
    let app = gpui_platform::application().with_assets(gpui_component_assets::Assets);

    app.run(move |cx| {
        gpui_component::init(cx);
        cx.set_menus(build_menus());

        let app_menus = build_menus().into_iter().map(Menu::owned).collect();
        GlobalState::global_mut(cx).set_app_menus(app_menus);

        let window_options = WindowOptions {
            titlebar: Some(TitleBar::title_bar_options()),
            window_bounds: Some(WindowBounds::centered(size(px(1240.), px(800.)), cx)),
            ..Default::default()
        };

        cx.spawn(async move |cx| {
            cx.open_window(window_options, move |window, cx| {
                let (document, startup_error) = match startup_result {
                    Ok(document) => (document, None),
                    Err(error) => (XtceDocument::untitled(), Some(error)),
                };
                let view = cx.new(|cx| XtceEditor::new(document, window, cx));
                if let Some(error) = startup_error {
                    window.push_notification(error, cx);
                }
                cx.new(|cx| Root::new(view, window, cx))
            })
            .expect("failed to open XTCE editor window");
        })
        .detach();
    });
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{
        AlgorithmChildKind, ElementKind, ElementSelection, ElementTree, IconName, StreamChildKind,
        XtceDocument, abbreviated_breadcrumb_label, breadcrumb_items, startup_document,
    };

    #[test]
    fn element_collections_are_directory_only_nodes() {
        for kind in [
            ElementKind::TelemetryParameterTypeSet,
            ElementKind::TelemetryParameterSet,
            ElementKind::TelemetryMetaData,
            ElementKind::CommandParameterTypeSet,
            ElementKind::CommandParameterSet,
            ElementKind::CommandMetaData,
            ElementKind::ArgumentTypeSet,
            ElementKind::MetaCommandSet,
            ElementKind::CommandContainerSet,
            ElementKind::TelemetryStreamSet,
            ElementKind::CommandStreamSet,
        ] {
            assert!(kind.is_directory_only());
        }
        assert!(ElementKind::ContainerSet.is_directory_only());
    }

    fn sample_document() -> xtce::SpaceSystem {
        xtce::from_str(include_str!("../../xtce/tests/fixtures/sample.xml"))
            .expect("sample.xml should decode")
    }

    #[test]
    fn every_element_kind_has_the_expected_breadcrumb_directories() {
        let cases: &[(ElementKind, &[&str])] = &[
            (ElementKind::SpaceSystem, &[]),
            (ElementKind::TelemetryMetaData, &[]),
            (
                ElementKind::TelemetryParameterTypeSet,
                &["TelemetryMetaData"],
            ),
            (
                ElementKind::TelemetryParameterType(0),
                &["TelemetryMetaData", "ParameterTypeSet"],
            ),
            (ElementKind::TelemetryParameterSet, &["TelemetryMetaData"]),
            (
                ElementKind::TelemetryParameter(0),
                &["TelemetryMetaData", "ParameterSet"],
            ),
            (ElementKind::ContainerSet, &["TelemetryMetaData"]),
            (
                ElementKind::SequenceContainer(0),
                &["TelemetryMetaData", "ContainerSet"],
            ),
            (ElementKind::MessageSet, &["TelemetryMetaData"]),
            (
                ElementKind::Message(0),
                &["TelemetryMetaData", "MessageSet"],
            ),
            (ElementKind::TelemetryStreamSet, &["TelemetryMetaData"]),
            (
                ElementKind::TelemetryFixedFrameStream(0),
                &["TelemetryMetaData", "StreamSet"],
            ),
            (
                ElementKind::TelemetryVariableFrameStream(0),
                &["TelemetryMetaData", "StreamSet"],
            ),
            (
                ElementKind::TelemetryCustomStream(0),
                &["TelemetryMetaData", "StreamSet"],
            ),
            (ElementKind::TelemetryAlgorithmSet, &["TelemetryMetaData"]),
            (
                ElementKind::TelemetryCustomAlgorithm(0),
                &["TelemetryMetaData", "AlgorithmSet"],
            ),
            (
                ElementKind::TelemetryMathAlgorithm(0),
                &["TelemetryMetaData", "AlgorithmSet"],
            ),
            (ElementKind::CommandMetaData, &[]),
            (ElementKind::CommandParameterTypeSet, &["CommandMetaData"]),
            (
                ElementKind::CommandParameterType(0),
                &["CommandMetaData", "ParameterTypeSet"],
            ),
            (ElementKind::CommandParameterSet, &["CommandMetaData"]),
            (
                ElementKind::CommandParameter(0),
                &["CommandMetaData", "ParameterSet"],
            ),
            (ElementKind::ArgumentTypeSet, &["CommandMetaData"]),
            (
                ElementKind::ArgumentType(0),
                &["CommandMetaData", "ArgumentTypeSet"],
            ),
            (ElementKind::MetaCommandSet, &["CommandMetaData"]),
            (
                ElementKind::MetaCommand(0),
                &["CommandMetaData", "MetaCommandSet"],
            ),
            (ElementKind::CommandContainerSet, &["CommandMetaData"]),
            (
                ElementKind::CommandContainer(0),
                &["CommandMetaData", "CommandContainerSet"],
            ),
            (ElementKind::CommandStreamSet, &["CommandMetaData"]),
            (
                ElementKind::CommandFixedFrameStream(0),
                &["CommandMetaData", "StreamSet"],
            ),
            (
                ElementKind::CommandVariableFrameStream(0),
                &["CommandMetaData", "StreamSet"],
            ),
            (
                ElementKind::CommandCustomStream(0),
                &["CommandMetaData", "StreamSet"],
            ),
            (ElementKind::CommandAlgorithmSet, &["CommandMetaData"]),
            (
                ElementKind::CommandCustomAlgorithm(0),
                &["CommandMetaData", "AlgorithmSet"],
            ),
            (
                ElementKind::CommandMathAlgorithm(0),
                &["CommandMetaData", "AlgorithmSet"],
            ),
            (ElementKind::ServiceSet, &[]),
            (ElementKind::Service(0), &["ServiceSet"]),
        ];

        for (kind, expected) in cases {
            assert_eq!(
                kind.breadcrumb_directories(),
                *expected,
                "unexpected breadcrumb directories for {kind:?}"
            );
        }
    }

    #[test]
    fn breadcrumb_includes_nested_systems_and_structural_directories() {
        let document = sample_document();
        let items = breadcrumb_items(
            &document,
            &ElementSelection {
                system_path: vec![0, 0],
                kind: ElementKind::TelemetryParameter(0),
            },
            "DraftParameter".to_owned(),
        );

        assert_eq!(
            items
                .iter()
                .map(|item| item.label.as_str())
                .collect::<Vec<_>>(),
            [
                "ExampleMission",
                "Payload",
                "Sensor",
                "TelemetryMetaData",
                "ParameterSet",
                "DraftParameter",
            ]
        );
        assert_eq!(items[0].system_path, Some(Vec::new()));
        assert_eq!(items[1].system_path, Some(vec![0]));
        assert_eq!(items[2].system_path, Some(vec![0, 0]));
        assert!(items[3..].iter().all(|item| item.system_path.is_none()));
    }

    #[test]
    fn current_space_system_uses_its_draft_name_in_the_breadcrumb() {
        let document = sample_document();
        let items = breadcrumb_items(
            &document,
            &ElementSelection {
                system_path: vec![0],
                kind: ElementKind::SpaceSystem,
            },
            "RenamedPayload".to_owned(),
        );

        assert_eq!(items[0].label, "ExampleMission");
        assert_eq!(items[1].label, "RenamedPayload");
        assert_eq!(items[1].system_path, Some(vec![0]));
    }

    #[test]
    fn long_breadcrumb_labels_are_abbreviated() {
        assert_eq!(
            abbreviated_breadcrumb_label("ABCDEFGHIJKLMNOPQRSTUVWXYZ123456789"),
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ12…"
        );
        assert_eq!(abbreviated_breadcrumb_label("ShortName"), "ShortName");
    }

    #[test]
    fn resolves_a_nested_space_system_from_its_tree_path() {
        let document = sample_document();

        assert_eq!(
            XtceDocument::system_at_path(&document, &[0, 0]).name,
            "Sensor"
        );
    }

    #[test]
    fn updates_only_the_space_system_at_the_selected_path() {
        let mut document = sample_document();

        XtceDocument::system_at_path_mut(&mut document, &[1]).asset_type =
            "control-center".to_owned();

        assert_eq!(document.name, "ExampleMission");
        assert_eq!(document.space_system[0].asset_type, "payload");
        assert_eq!(document.space_system[1].asset_type, "control-center");
        xtce::to_string(&document).expect("the edited document should encode");
    }

    #[test]
    fn serializes_a_file_with_an_xml_declaration() {
        let document = sample_document();

        let xml = XtceDocument::serialize(&document).expect("document should serialize");

        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n"));
        let decoded = xtce::from_str(&xml).expect("saved XML should decode");
        assert_eq!(decoded.name, document.name);
    }

    #[test]
    fn parses_an_opened_document_and_resets_selection_to_the_root() {
        let document = XtceDocument::from_xml(
            include_str!("../../xtce/tests/fixtures/sample.xml"),
            "opened.xml".to_owned(),
        )
        .expect("opened XML should decode");

        assert_eq!(document.file_name, "opened.xml");
        assert_eq!(document.root.name, "ExampleMission");
        assert_eq!(document.selection.kind, ElementKind::SpaceSystem);
        assert!(document.selection.system_path.is_empty());
        assert!(XtceDocument::from_xml("<invalid>", "invalid.xml".to_owned()).is_err());
    }

    #[test]
    fn opens_an_xtce_1_2_document_with_a_self_closing_header() {
        let xml = format!(
            r#"
            <xtce:SpaceSystem
                xmlns:xtce="{}"
                name="LegacyMission"
            >
              <xtce:Header version="0.0.0" validationStatus="Draft" />
              <xtce:TelemetryMetaData>
                <xtce:ParameterTypeSet />
                <xtce:ParameterSet />
              </xtce:TelemetryMetaData>
            </xtce:SpaceSystem>
            "#,
            XtceDocument::XTCE_1_2_NAMESPACE
        );

        let document = XtceDocument::from_xml(&xml, "legacy.xtce".to_owned())
            .expect("XTCE 1.2 XML should open through compatibility conversion");

        assert_eq!(document.root.name, "LegacyMission");
        assert_eq!(
            document.root.header.as_ref().unwrap().version.as_deref(),
            Some("0.0.0")
        );
    }

    #[test]
    fn creates_a_minimal_untitled_document() {
        let document = XtceDocument::untitled();

        assert_eq!(document.file_name, "untitled.xml");
        assert_eq!(document.root.name, "NewSpaceSystem");
        assert_eq!(document.selection.kind, ElementKind::SpaceSystem);
        assert!(document.selection.system_path.is_empty());
        XtceDocument::serialize(&document.root).expect("new document should serialize");
    }

    #[test]
    fn startup_without_a_file_uses_an_empty_document() {
        let document = startup_document(None).expect("empty startup should succeed");

        assert_eq!(document.file_name, "untitled.xml");
        assert_eq!(document.root.name, "NewSpaceSystem");
        assert!(document.root.telemetry_meta_data.is_none());
        assert!(document.root.command_meta_data.is_none());
    }

    #[test]
    fn adds_each_metadata_element_only_once() {
        let mut document = XtceDocument::untitled().root;

        assert!(XtceDocument::add_metadata(
            &mut document,
            ElementKind::TelemetryMetaData
        ));
        assert!(XtceDocument::add_metadata(
            &mut document,
            ElementKind::CommandMetaData
        ));
        assert!(XtceDocument::add_metadata(
            &mut document,
            ElementKind::ServiceSet
        ));
        assert!(!XtceDocument::add_metadata(
            &mut document,
            ElementKind::TelemetryMetaData
        ));
        assert!(!XtceDocument::add_metadata(
            &mut document,
            ElementKind::CommandMetaData
        ));
        assert!(!XtceDocument::add_metadata(
            &mut document,
            ElementKind::ServiceSet
        ));
        assert!(document.telemetry_meta_data.is_some());
        assert!(document.command_meta_data.is_some());
        assert!(document.service_set.is_some());
        XtceDocument::serialize(&document).expect("metadata document should serialize");
    }

    #[test]
    fn service_set_is_a_directory_of_service_elements() {
        let mut root = XtceDocument::untitled().root;
        assert!(XtceDocument::add_metadata(
            &mut root,
            ElementKind::ServiceSet
        ));

        let added = XtceDocument::add_collection_item(&mut root, ElementKind::ServiceSet);
        assert_eq!(added, Some(ElementKind::Service(0)));
        assert_eq!(
            root.service_set.as_ref().unwrap().service[0].name,
            "Service1"
        );

        let mut nodes = Vec::new();
        XtceDocument::collect_tree_nodes(&root, &mut Vec::new(), 0, &mut nodes);
        let set = nodes
            .iter()
            .find(|node| node.selection.kind == ElementKind::ServiceSet)
            .expect("service set tree node");
        assert!(set.has_children);
        assert!(nodes.iter().any(|node| {
            node.selection.kind == ElementKind::Service(0) && node.label == "Service1"
        }));

        assert!(XtceDocument::delete_element(
            &mut root,
            &ElementSelection {
                system_path: Vec::new(),
                kind: ElementKind::Service(0),
            },
        ));
        assert!(root.service_set.as_ref().unwrap().service.is_empty());
    }

    #[test]
    fn telemetry_metadata_always_has_parameter_set_tree_nodes() {
        let mut document = XtceDocument::untitled().root;
        assert!(XtceDocument::add_metadata(
            &mut document,
            ElementKind::TelemetryMetaData
        ));
        let mut nodes = Vec::new();

        XtceDocument::collect_tree_nodes(&document, &mut Vec::new(), 0, &mut nodes);

        assert!(nodes.iter().any(|node| {
            node.selection.kind == ElementKind::TelemetryParameterTypeSet
                && node.selection.system_path.is_empty()
        }));
        assert!(nodes.iter().any(|node| {
            node.selection.kind == ElementKind::TelemetryParameterSet
                && node.selection.system_path.is_empty()
        }));
    }

    #[test]
    fn adding_to_virtual_telemetry_sets_materializes_them() {
        let mut document = XtceDocument::untitled().root;
        assert!(XtceDocument::add_metadata(
            &mut document,
            ElementKind::TelemetryMetaData
        ));

        let parameter_type = XtceDocument::add_collection_item(
            &mut document,
            ElementKind::TelemetryParameterTypeSet,
        );
        let parameter =
            XtceDocument::add_collection_item(&mut document, ElementKind::TelemetryParameterSet);

        assert_eq!(parameter_type, Some(ElementKind::TelemetryParameterType(0)));
        assert_eq!(parameter, Some(ElementKind::TelemetryParameter(0)));
        let metadata = document
            .telemetry_meta_data
            .as_ref()
            .expect("telemetry metadata");
        assert_eq!(
            metadata
                .parameter_type_set
                .as_ref()
                .expect("parameter type set")
                .content
                .len(),
            1
        );
        assert_eq!(
            metadata
                .parameter_set
                .as_ref()
                .expect("parameter set")
                .content
                .len(),
            1
        );
        XtceDocument::serialize(&document).expect("materialized sets should serialize");
    }

    #[test]
    fn adding_a_message_materializes_its_set_and_tree_node() {
        let mut document = XtceDocument::untitled().root;
        assert!(XtceDocument::add_metadata(
            &mut document,
            ElementKind::TelemetryMetaData
        ));

        let message = XtceDocument::add_collection_item(&mut document, ElementKind::MessageSet);

        assert_eq!(message, Some(ElementKind::Message(0)));
        let set = document
            .telemetry_meta_data
            .as_ref()
            .and_then(|metadata| metadata.message_set.as_ref())
            .expect("message set");
        assert_eq!(set.message.len(), 1);
        assert_eq!(set.message[0].name, "Message1");
        assert!(set.message[0].container_ref.container_ref.is_empty());
        let xtce::MatchCriteriaType::Comparison(comparison) = &set.message[0].match_criteria else {
            panic!("new messages should start with one comparison");
        };
        assert!(comparison.parameter_ref.is_empty());

        let mut nodes = Vec::new();
        XtceDocument::collect_tree_nodes(&document, &mut Vec::new(), 0, &mut nodes);
        assert!(
            nodes.iter().any(|node| {
                node.selection.kind == ElementKind::MessageSet && node.has_children
            })
        );
        assert!(nodes.iter().any(|node| {
            node.selection.kind == ElementKind::Message(0) && node.label == "Message1"
        }));
    }

    #[test]
    fn message_survives_xtce_xml_round_trip() {
        let mut root = XtceDocument::untitled().root;
        assert!(XtceDocument::add_metadata(
            &mut root,
            ElementKind::TelemetryMetaData
        ));
        assert_eq!(
            XtceDocument::add_collection_item(&mut root, ElementKind::MessageSet),
            Some(ElementKind::Message(0))
        );
        let message = &mut root
            .telemetry_meta_data
            .as_mut()
            .unwrap()
            .message_set
            .as_mut()
            .unwrap()
            .message[0];
        message.name = "HousekeepingMessage".to_owned();
        message.container_ref.container_ref = "/Vehicle/Housekeeping".to_owned();

        let xml = XtceDocument::serialize(&root).expect("message document should serialize");
        let decoded =
            XtceDocument::from_xml(&xml, "messages.xml".to_owned()).expect("message should decode");
        let decoded_message = &decoded
            .root
            .telemetry_meta_data
            .as_ref()
            .unwrap()
            .message_set
            .as_ref()
            .unwrap()
            .message[0];

        assert_eq!(decoded_message.name, "HousekeepingMessage");
        assert_eq!(
            decoded_message.container_ref.container_ref,
            "/Vehicle/Housekeeping"
        );
    }

    #[test]
    fn adds_fixed_frame_streams_to_telemetry_and_command_metadata() {
        let mut root = XtceDocument::untitled().root;
        assert!(XtceDocument::add_metadata(
            &mut root,
            ElementKind::TelemetryMetaData
        ));
        assert!(XtceDocument::add_metadata(
            &mut root,
            ElementKind::CommandMetaData
        ));

        assert_eq!(
            XtceDocument::add_collection_item(&mut root, ElementKind::TelemetryStreamSet),
            Some(ElementKind::TelemetryFixedFrameStream(0))
        );
        assert_eq!(
            XtceDocument::add_collection_item(&mut root, ElementKind::CommandStreamSet),
            Some(ElementKind::CommandFixedFrameStream(0))
        );

        let telemetry_stream = &root
            .telemetry_meta_data
            .as_ref()
            .unwrap()
            .stream_set
            .as_ref()
            .unwrap()
            .content[0];
        let command_stream = &root
            .command_meta_data
            .as_ref()
            .unwrap()
            .stream_set
            .as_ref()
            .unwrap()
            .content[0];
        assert!(matches!(
            telemetry_stream,
            xtce::StreamSetTypeContent::FixedFrameStream(stream)
                if stream.name == "FixedFrameStream1"
        ));
        assert!(matches!(
            command_stream,
            xtce::StreamSetTypeContent::FixedFrameStream(stream)
                if stream.name == "FixedFrameStream1"
        ));

        let mut nodes = Vec::new();
        XtceDocument::collect_tree_nodes(&root, &mut Vec::new(), 0, &mut nodes);
        assert!(nodes.iter().any(|node| {
            node.selection.kind == ElementKind::TelemetryFixedFrameStream(0)
                && node.label == "FixedFrameStream1"
        }));
        assert!(nodes.iter().any(|node| {
            node.selection.kind == ElementKind::CommandFixedFrameStream(0)
                && node.label == "FixedFrameStream1"
        }));
        XtceDocument::serialize(&root).expect("fixed frame streams should serialize");
    }

    #[test]
    fn adds_variable_frame_streams_to_telemetry_and_command_metadata() {
        let mut root = XtceDocument::untitled().root;
        assert!(XtceDocument::add_metadata(
            &mut root,
            ElementKind::TelemetryMetaData
        ));
        assert!(XtceDocument::add_metadata(
            &mut root,
            ElementKind::CommandMetaData
        ));

        assert_eq!(
            XtceDocument::add_stream_item(
                &mut root,
                ElementKind::TelemetryStreamSet,
                StreamChildKind::Variable,
            ),
            Some(ElementKind::TelemetryVariableFrameStream(0))
        );
        assert_eq!(
            XtceDocument::add_stream_item(
                &mut root,
                ElementKind::CommandStreamSet,
                StreamChildKind::Variable,
            ),
            Some(ElementKind::CommandVariableFrameStream(0))
        );

        let telemetry_stream = &root
            .telemetry_meta_data
            .as_ref()
            .unwrap()
            .stream_set
            .as_ref()
            .unwrap()
            .content[0];
        assert!(matches!(
            telemetry_stream,
            xtce::StreamSetTypeContent::VariableFrameStream(stream)
                if stream.name == "VariableFrameStream1"
        ));

        let mut nodes = Vec::new();
        XtceDocument::collect_tree_nodes(&root, &mut Vec::new(), 0, &mut nodes);
        assert!(nodes.iter().any(|node| {
            node.selection.kind == ElementKind::TelemetryVariableFrameStream(0)
                && node.label == "VariableFrameStream1"
        }));
        assert!(nodes.iter().any(|node| {
            node.selection.kind == ElementKind::CommandVariableFrameStream(0)
                && node.label == "VariableFrameStream1"
        }));
        XtceDocument::serialize(&root).expect("variable frame streams should serialize");
    }

    #[test]
    fn adds_custom_streams_to_telemetry_and_command_metadata() {
        let mut root = XtceDocument::untitled().root;
        assert!(XtceDocument::add_metadata(
            &mut root,
            ElementKind::TelemetryMetaData
        ));
        assert!(XtceDocument::add_metadata(
            &mut root,
            ElementKind::CommandMetaData
        ));

        assert_eq!(
            XtceDocument::add_stream_item(
                &mut root,
                ElementKind::TelemetryStreamSet,
                StreamChildKind::Custom,
            ),
            Some(ElementKind::TelemetryCustomStream(0))
        );
        assert_eq!(
            XtceDocument::add_stream_item(
                &mut root,
                ElementKind::CommandStreamSet,
                StreamChildKind::Custom,
            ),
            Some(ElementKind::CommandCustomStream(0))
        );

        let mut nodes = Vec::new();
        XtceDocument::collect_tree_nodes(&root, &mut Vec::new(), 0, &mut nodes);
        assert!(nodes.iter().any(|node| {
            node.selection.kind == ElementKind::TelemetryCustomStream(0)
                && node.label == "CustomStream1"
        }));
        assert!(nodes.iter().any(|node| {
            node.selection.kind == ElementKind::CommandCustomStream(0)
                && node.label == "CustomStream1"
        }));

        let xml = XtceDocument::serialize(&root).expect("custom streams should serialize");
        let reopened =
            XtceDocument::from_xml(&xml, "custom-stream.xml".to_owned()).expect("should reopen");
        assert!(matches!(
            reopened
                .root
                .telemetry_meta_data
                .and_then(|metadata| metadata.stream_set)
                .and_then(|set| set.content.into_iter().next()),
            Some(xtce::StreamSetTypeContent::CustomStream(stream))
                if stream.name == "CustomStream1"
        ));
    }

    #[test]
    fn adds_custom_algorithms_to_telemetry_and_command_metadata() {
        let mut root = XtceDocument::untitled().root;
        assert!(XtceDocument::add_metadata(
            &mut root,
            ElementKind::TelemetryMetaData
        ));
        assert!(XtceDocument::add_metadata(
            &mut root,
            ElementKind::CommandMetaData
        ));

        assert_eq!(
            XtceDocument::add_collection_item(&mut root, ElementKind::TelemetryAlgorithmSet),
            Some(ElementKind::TelemetryCustomAlgorithm(0))
        );
        assert_eq!(
            XtceDocument::add_collection_item(&mut root, ElementKind::CommandAlgorithmSet),
            Some(ElementKind::CommandCustomAlgorithm(0))
        );

        let mut nodes = Vec::new();
        XtceDocument::collect_tree_nodes(&root, &mut Vec::new(), 0, &mut nodes);
        assert!(nodes.iter().any(|node| {
            node.selection.kind == ElementKind::TelemetryCustomAlgorithm(0)
                && node.label == "CustomAlgorithm1"
        }));
        assert!(nodes.iter().any(|node| {
            node.selection.kind == ElementKind::CommandCustomAlgorithm(0)
                && node.label == "CustomAlgorithm1"
        }));

        let xml = XtceDocument::serialize(&root).expect("custom algorithms should serialize");
        let reopened =
            XtceDocument::from_xml(&xml, "custom-algorithm.xml".to_owned()).expect("should reopen");
        assert!(matches!(
            reopened
                .root
                .telemetry_meta_data
                .and_then(|metadata| metadata.algorithm_set)
                .and_then(|set| set.content.into_iter().next()),
            Some(xtce::AlgorithmSetTypeContent::CustomAlgorithm(algorithm))
                if algorithm.name == "CustomAlgorithm1"
        ));
    }

    #[test]
    fn adds_math_algorithms_to_telemetry_and_command_metadata() {
        let mut root = XtceDocument::untitled().root;
        assert!(XtceDocument::add_metadata(
            &mut root,
            ElementKind::TelemetryMetaData
        ));
        assert!(XtceDocument::add_metadata(
            &mut root,
            ElementKind::CommandMetaData
        ));

        assert_eq!(
            XtceDocument::add_algorithm_item(
                &mut root,
                ElementKind::TelemetryAlgorithmSet,
                AlgorithmChildKind::Math,
            ),
            Some(ElementKind::TelemetryMathAlgorithm(0))
        );
        assert_eq!(
            XtceDocument::add_algorithm_item(
                &mut root,
                ElementKind::CommandAlgorithmSet,
                AlgorithmChildKind::Math,
            ),
            Some(ElementKind::CommandMathAlgorithm(0))
        );

        let mut nodes = Vec::new();
        XtceDocument::collect_tree_nodes(&root, &mut Vec::new(), 0, &mut nodes);
        assert!(nodes.iter().any(|node| {
            node.selection.kind == ElementKind::TelemetryMathAlgorithm(0)
                && node.label == "MathAlgorithm1"
        }));
        assert!(nodes.iter().any(|node| {
            node.selection.kind == ElementKind::CommandMathAlgorithm(0)
                && node.label == "MathAlgorithm1"
        }));

        let xml = XtceDocument::serialize(&root).expect("math algorithms should serialize");
        let reopened =
            XtceDocument::from_xml(&xml, "math-algorithm.xml".to_owned()).expect("should reopen");
        assert!(matches!(
            reopened
                .root
                .telemetry_meta_data
                .and_then(|metadata| metadata.algorithm_set)
                .and_then(|set| set.content.into_iter().next()),
            Some(xtce::AlgorithmSetTypeContent::MathAlgorithm(algorithm))
                if algorithm.name == "MathAlgorithm1"
        ));
    }

    #[test]
    fn adds_argument_types_and_meta_commands_to_command_metadata() {
        let mut document = XtceDocument::untitled().root;
        assert!(XtceDocument::add_metadata(
            &mut document,
            ElementKind::CommandMetaData
        ));

        let argument_type =
            XtceDocument::add_collection_item(&mut document, ElementKind::ArgumentTypeSet);
        let meta_command =
            XtceDocument::add_collection_item(&mut document, ElementKind::MetaCommandSet);

        assert_eq!(argument_type, Some(ElementKind::ArgumentType(0)));
        assert_eq!(meta_command, Some(ElementKind::MetaCommand(0)));
        let metadata = document
            .command_meta_data
            .as_ref()
            .expect("command metadata");
        assert_eq!(
            XtceDocument::argument_type_label(
                &metadata
                    .argument_type_set
                    .as_ref()
                    .expect("argument type set")
                    .content[0]
            ),
            "ArgumentType1"
        );
        assert_eq!(
            XtceDocument::meta_command_label(
                &metadata
                    .meta_command_set
                    .as_ref()
                    .expect("meta command set")
                    .content[0]
            ),
            "MetaCommand1"
        );
        let mut nodes = Vec::new();
        XtceDocument::collect_tree_nodes(&document, &mut Vec::new(), 0, &mut nodes);
        assert!(nodes.iter().any(|node| {
            node.selection.kind == ElementKind::ArgumentType(0) && node.label == "ArgumentType1"
        }));
        assert!(nodes.iter().any(|node| {
            node.selection.kind == ElementKind::MetaCommand(0) && node.label == "MetaCommand1"
        }));
        XtceDocument::serialize(&document).expect("command elements should serialize");
    }

    #[test]
    fn adds_sequence_containers_to_telemetry_metadata() {
        let mut document = XtceDocument::untitled().root;
        assert!(XtceDocument::add_metadata(
            &mut document,
            ElementKind::TelemetryMetaData
        ));

        let first = XtceDocument::add_collection_item(&mut document, ElementKind::ContainerSet);
        let second = XtceDocument::add_collection_item(&mut document, ElementKind::ContainerSet);

        assert_eq!(first, Some(ElementKind::SequenceContainer(0)));
        assert_eq!(second, Some(ElementKind::SequenceContainer(1)));
        let containers = &document
            .telemetry_meta_data
            .as_ref()
            .expect("telemetry metadata")
            .container_set
            .as_ref()
            .expect("container set")
            .content;
        assert_eq!(
            XtceDocument::sequence_container_label(&containers[0]),
            "SequenceContainer1"
        );
        assert_eq!(
            XtceDocument::sequence_container_label(&containers[1]),
            "SequenceContainer2"
        );
        let mut nodes = Vec::new();
        XtceDocument::collect_tree_nodes(&document, &mut Vec::new(), 0, &mut nodes);
        assert!(nodes.iter().any(|node| {
            node.selection.kind == ElementKind::SequenceContainer(0)
                && node.label == "SequenceContainer1"
        }));
        let xml = XtceDocument::serialize(&document).expect("container set should serialize");
        assert!(xml.contains("idlePattern=\"0\""));
    }

    #[test]
    fn adds_command_containers_to_command_metadata() {
        let mut document = XtceDocument::untitled().root;
        assert!(XtceDocument::add_metadata(
            &mut document,
            ElementKind::CommandMetaData
        ));

        let first =
            XtceDocument::add_collection_item(&mut document, ElementKind::CommandContainerSet);
        let second =
            XtceDocument::add_collection_item(&mut document, ElementKind::CommandContainerSet);

        assert_eq!(first, Some(ElementKind::CommandContainer(0)));
        assert_eq!(second, Some(ElementKind::CommandContainer(1)));
        let containers = &document
            .command_meta_data
            .as_ref()
            .expect("command metadata")
            .command_container_set
            .as_ref()
            .expect("command container set")
            .command_container;
        assert_eq!(containers[0].name, "CommandContainer1");
        assert_eq!(containers[1].name, "CommandContainer2");

        let mut nodes = Vec::new();
        XtceDocument::collect_tree_nodes(&document, &mut Vec::new(), 0, &mut nodes);
        assert!(nodes.iter().any(|node| {
            node.selection.kind == ElementKind::CommandContainer(0)
                && node.label == "CommandContainer1"
        }));

        let xml =
            XtceDocument::serialize(&document).expect("command container set should serialize");
        let reopened = XtceDocument::from_xml(&xml, "command-containers.xml".to_owned())
            .expect("command container set should reopen");
        assert_eq!(
            reopened
                .root
                .command_meta_data
                .and_then(|metadata| metadata.command_container_set)
                .expect("command container set")
                .command_container
                .len(),
            2
        );
    }

    #[test]
    fn adds_uniquely_named_child_space_systems() {
        let mut document = XtceDocument::untitled().root;

        let first = XtceDocument::add_space_system(&mut document);
        let second = XtceDocument::add_space_system(&mut document);

        assert_eq!(first, 0);
        assert_eq!(second, 1);
        assert_eq!(document.space_system[0].name, "SpaceSystem1");
        assert_eq!(document.space_system[1].name, "SpaceSystem2");
        XtceDocument::serialize(&document).expect("nested document should serialize");
    }

    #[test]
    fn tree_contains_present_metadata_elements_for_each_space_system() {
        let document = sample_document();
        let mut nodes = Vec::new();
        XtceDocument::collect_tree_nodes(&document, &mut Vec::new(), 0, &mut nodes);

        assert!(nodes.iter().any(|node| {
            node.selection.system_path.is_empty()
                && node.selection.kind == ElementKind::TelemetryMetaData
        }));
        assert!(nodes.iter().any(|node| {
            node.selection.system_path == [0]
                && node.selection.kind == ElementKind::TelemetryParameterSet
        }));
        assert!(nodes.iter().any(|node| {
            node.selection.system_path == [0]
                && matches!(node.selection.kind, ElementKind::TelemetryParameter(_))
                && node.label == "SampleCount"
        }));
        assert!(nodes.iter().any(|node| {
            node.selection.system_path.is_empty()
                && matches!(node.selection.kind, ElementKind::TelemetryParameterType(_))
                && node.label == "OperationalFlagType"
        }));
        assert!(
            !nodes
                .iter()
                .any(|node| node.label.ends_with("DataEncoding"))
        );
        assert!(nodes.iter().any(|node| {
            node.selection.system_path == [0, 0]
                && node.selection.kind == ElementKind::TelemetryMetaData
        }));
        assert!(nodes.iter().any(|node| {
            node.selection.system_path == [1] && node.selection.kind == ElementKind::CommandMetaData
        }));
    }

    #[test]
    fn empty_command_metadata_keeps_addable_set_directories_visible() {
        let document = sample_document();
        let mut nodes = Vec::new();
        XtceDocument::collect_tree_nodes(&document, &mut Vec::new(), 0, &mut nodes);
        let command_metadata = nodes
            .iter()
            .find(|node| {
                node.selection.system_path.is_empty()
                    && node.selection.kind == ElementKind::CommandMetaData
            })
            .expect("root command metadata");

        assert!(command_metadata.has_children);
        assert!(matches!(
            command_metadata.selection.kind.tree_icon(false),
            IconName::SquareTerminal
        ));
        assert!(nodes.iter().any(|node| {
            node.selection.system_path.is_empty()
                && node.selection.kind == ElementKind::ArgumentTypeSet
        }));
        assert!(nodes.iter().any(|node| {
            node.selection.system_path.is_empty()
                && node.selection.kind == ElementKind::MetaCommandSet
        }));
        assert!(nodes.iter().any(|node| {
            node.selection.system_path.is_empty()
                && node.selection.kind == ElementKind::CommandParameterTypeSet
        }));
        assert!(nodes.iter().any(|node| {
            node.selection.system_path.is_empty()
                && node.selection.kind == ElementKind::CommandParameterSet
        }));
    }

    #[test]
    fn collapsing_a_tree_node_hides_only_its_descendants() {
        let document = sample_document();
        let mut nodes = Vec::new();
        XtceDocument::collect_tree_nodes(&document, &mut Vec::new(), 0, &mut nodes);
        let collapsed = HashSet::from([ElementSelection {
            system_path: Vec::new(),
            kind: ElementKind::TelemetryMetaData,
        }]);

        let visible = ElementTree::visible_nodes(nodes, &collapsed);

        assert!(visible.iter().any(|node| {
            node.selection.kind == ElementKind::TelemetryMetaData
                && node.selection.system_path.is_empty()
        }));
        assert!(!visible.iter().any(|node| {
            node.selection.kind == ElementKind::TelemetryParameterTypeSet
                && node.selection.system_path.is_empty()
        }));
        assert!(visible.iter().any(|node| {
            node.selection.kind == ElementKind::CommandMetaData
                && node.selection.system_path.is_empty()
        }));
    }

    #[test]
    fn collapsing_the_root_hides_the_entire_document_subtree() {
        let document = sample_document();
        let mut nodes = Vec::new();
        XtceDocument::collect_tree_nodes(&document, &mut Vec::new(), 0, &mut nodes);
        let collapsed = HashSet::from([ElementSelection {
            system_path: Vec::new(),
            kind: ElementKind::SpaceSystem,
        }]);

        let visible = ElementTree::visible_nodes(nodes, &collapsed);

        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].selection.kind, ElementKind::SpaceSystem);
    }

    #[test]
    fn a_selection_inside_a_collapsed_set_is_not_revealed_during_rebuild() {
        let document = sample_document();
        let mut nodes = Vec::new();
        XtceDocument::collect_tree_nodes(&document, &mut Vec::new(), 0, &mut nodes);
        let parent = ElementSelection {
            system_path: Vec::new(),
            kind: ElementKind::TelemetryParameterTypeSet,
        };
        let hidden_selection = ElementSelection {
            system_path: Vec::new(),
            kind: ElementKind::TelemetryParameterType(0),
        };
        let collapsed = HashSet::from([parent.clone()]);
        let mut index = 0;
        let items = ElementTree::build_items(
            &nodes,
            &mut index,
            0,
            "",
            &collapsed,
            &HashSet::new(),
            Some(&hidden_selection),
        );
        let hidden_parameter_type = ElementTree::node_id(&hidden_selection);
        let visible_parameter = ElementTree::node_id(&ElementSelection {
            system_path: Vec::new(),
            kind: ElementKind::TelemetryParameter(0),
        });

        assert!(
            ElementTree::find_visible_item(&items, &hidden_parameter_type).is_none(),
            "a hidden selection must not be passed to TreeState, which would reopen its ancestors"
        );
        assert!(
            ElementTree::find_visible_item(&items, &visible_parameter).is_some(),
            "a selection in an expanded sibling set should remain selectable"
        );
        assert_eq!(
            ElementTree::find_hidden_selection_ancestor(&items, &hidden_parameter_type)
                .map(|item| item.id.clone()),
            Some(ElementTree::node_id(&parent))
        );
    }

    #[test]
    fn a_set_manually_collapsed_during_filtering_stays_closed_during_rebuild() {
        let document = sample_document();
        let mut nodes = Vec::new();
        XtceDocument::collect_tree_nodes(&document, &mut Vec::new(), 0, &mut nodes);
        let collapsed = ElementTree::collapsed_by_default(&document);
        let parameter_type_set = ElementSelection {
            system_path: Vec::new(),
            kind: ElementKind::TelemetryParameterTypeSet,
        };
        let parameter_type_id = ElementTree::node_id(&ElementSelection {
            system_path: Vec::new(),
            kind: ElementKind::TelemetryParameterType(0),
        });
        let parameter_id = ElementTree::node_id(&ElementSelection {
            system_path: Vec::new(),
            kind: ElementKind::TelemetryParameter(0),
        });

        let mut index = 0;
        let initially_filtered = ElementTree::build_items(
            &nodes,
            &mut index,
            0,
            "operationalflag",
            &collapsed,
            &HashSet::new(),
            None,
        );
        assert!(
            ElementTree::find_visible_item(&initially_filtered, &parameter_type_id).is_some(),
            "matching descendants should initially be revealed"
        );
        assert!(ElementTree::find_visible_item(&initially_filtered, &parameter_id).is_some());

        let filter_collapsed = HashSet::from([parameter_type_set.clone()]);
        index = 0;
        let rebuilt = ElementTree::build_items(
            &nodes,
            &mut index,
            0,
            "operationalflag",
            &collapsed,
            &filter_collapsed,
            None,
        );
        let parameter_type_set_id = ElementTree::node_id(&parameter_type_set);
        let parameter_type_set_item =
            ElementTree::find_visible_item(&rebuilt, &parameter_type_set_id)
                .expect("the collapsed matching set remains visible");

        assert!(!parameter_type_set_item.is_expanded());
        assert!(ElementTree::find_visible_item(&rebuilt, &parameter_type_id).is_none());
        assert!(ElementTree::find_visible_item(&rebuilt, &parameter_id).is_some());
    }

    #[test]
    fn every_parent_node_is_collapsed_by_default() {
        let document = sample_document();
        let mut nodes = Vec::new();
        XtceDocument::collect_tree_nodes(&document, &mut Vec::new(), 0, &mut nodes);

        let collapsed = ElementTree::collapsed_by_default(&document);

        assert!(
            nodes
                .iter()
                .filter(|node| node.has_children)
                .all(|node| collapsed.contains(&node.selection))
        );
        assert!(
            nodes
                .iter()
                .filter(|node| !node.has_children)
                .all(|node| !collapsed.contains(&node.selection))
        );
    }

    #[test]
    fn tree_state_items_preserve_the_document_hierarchy() {
        let document = sample_document();
        let mut nodes = Vec::new();
        XtceDocument::collect_tree_nodes(&document, &mut Vec::new(), 0, &mut nodes);
        let collapsed = HashSet::new();
        let mut index = 0;

        let items =
            ElementTree::build_items(&nodes, &mut index, 0, "", &collapsed, &HashSet::new(), None);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label.as_ref(), "ExampleMission");
        assert!(
            items[0]
                .children
                .iter()
                .any(|item| item.label.as_ref() == "Payload")
        );
        assert!(
            items[0]
                .children
                .iter()
                .any(|item| item.label.as_ref() == "GroundSegment")
        );
    }

    #[test]
    fn tree_search_keeps_matching_items_and_their_ancestors() {
        fn contains_label(items: &[gpui_component::tree::TreeItem], label: &str) -> bool {
            items
                .iter()
                .any(|item| item.label.as_ref() == label || contains_label(&item.children, label))
        }

        let document = sample_document();
        let mut nodes = Vec::new();
        XtceDocument::collect_tree_nodes(&document, &mut Vec::new(), 0, &mut nodes);
        let collapsed = ElementTree::collapsed_by_default(&document);
        let mut index = 0;

        let items = ElementTree::build_items(
            &nodes,
            &mut index,
            0,
            "samplecount",
            &collapsed,
            &HashSet::new(),
            None,
        );

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label.as_ref(), "ExampleMission");
        assert!(items[0].is_expanded());
        assert!(contains_label(&items, "SampleCount"));
        assert!(!contains_label(&items, "OperationalFlag"));
    }

    #[test]
    fn tree_search_keeps_the_current_selection_for_context() {
        let document = sample_document();
        let mut nodes = Vec::new();
        XtceDocument::collect_tree_nodes(&document, &mut Vec::new(), 0, &mut nodes);
        let collapsed = ElementTree::collapsed_by_default(&document);
        let selection = ElementSelection {
            system_path: Vec::new(),
            kind: ElementKind::TelemetryParameter(0),
        };
        let selected_id = ElementTree::node_id(&selection);
        let matching_id = ElementTree::node_id(
            &nodes
                .iter()
                .find(|node| node.label == "SampleCount")
                .expect("sample fixture contains SampleCount")
                .selection,
        );
        let mut index = 0;

        assert!(!ElementTree::selection_is_filter_context(
            &nodes,
            &selection,
            "samplecount"
        ));
        assert!(ElementTree::selection_is_filter_context(
            &nodes,
            &ElementSelection {
                system_path: Vec::new(),
                kind: ElementKind::SpaceSystem,
            },
            "samplecount"
        ));

        let items = ElementTree::build_items(
            &nodes,
            &mut index,
            0,
            "samplecount",
            &collapsed,
            &HashSet::new(),
            Some(&selection),
        );

        assert!(
            ElementTree::find_visible_item(&items, &selected_id).is_some(),
            "renamed or post-delete selections should remain visible as filter context"
        );
        assert!(
            ElementTree::find_visible_item(&items, &matching_id).is_some(),
            "matching search results should remain visible beside the current selection"
        );
    }

    #[test]
    fn adding_to_a_parameter_type_set_creates_a_uniquely_named_type() {
        let mut document = sample_document();
        let original_len = document
            .telemetry_meta_data
            .as_ref()
            .and_then(|metadata| metadata.parameter_type_set.as_ref())
            .expect("telemetry parameter type set")
            .content
            .len();

        let added = XtceDocument::add_collection_item(
            &mut document,
            ElementKind::TelemetryParameterTypeSet,
        );

        assert_eq!(
            added,
            Some(ElementKind::TelemetryParameterType(original_len))
        );
        let set = document
            .telemetry_meta_data
            .as_ref()
            .and_then(|metadata| metadata.parameter_type_set.as_ref())
            .expect("telemetry parameter type set");
        assert_eq!(set.content.len(), original_len + 1);
        assert_eq!(
            XtceDocument::parameter_type_label(&set.content[original_len]),
            "ParameterType1"
        );
    }

    #[test]
    fn adding_to_a_parameter_set_uses_an_existing_type_reference() {
        let mut document = sample_document();
        let original_len = document
            .telemetry_meta_data
            .as_ref()
            .and_then(|metadata| metadata.parameter_set.as_ref())
            .expect("telemetry parameter set")
            .content
            .len();

        let added =
            XtceDocument::add_collection_item(&mut document, ElementKind::TelemetryParameterSet);

        assert_eq!(added, Some(ElementKind::TelemetryParameter(original_len)));
        let set = document
            .telemetry_meta_data
            .as_ref()
            .and_then(|metadata| metadata.parameter_set.as_ref())
            .expect("telemetry parameter set");
        let xtce::ParameterSetTypeContent::Parameter(parameter) = &set.content[original_len] else {
            panic!("expected a Parameter");
        };
        assert_eq!(parameter.name, "Parameter1");
        assert_eq!(parameter.parameter_type_ref, "OperationalFlagType");
    }

    #[test]
    fn deleting_the_last_collection_item_omits_its_empty_set() {
        let mut root = xtce::SpaceSystem::new("Root");
        assert!(XtceDocument::add_metadata(
            &mut root,
            ElementKind::TelemetryMetaData
        ));
        assert_eq!(
            XtceDocument::add_collection_item(&mut root, ElementKind::TelemetryParameterTypeSet),
            Some(ElementKind::TelemetryParameterType(0))
        );

        assert!(XtceDocument::delete_element(
            &mut root,
            &ElementSelection {
                system_path: Vec::new(),
                kind: ElementKind::TelemetryParameterType(0),
            }
        ));

        assert!(
            root.telemetry_meta_data
                .as_ref()
                .expect("telemetry metadata")
                .parameter_type_set
                .is_none()
        );
    }

    #[test]
    fn reference_check_reports_the_named_element_using_a_parameter_type() {
        let mut root = xtce::SpaceSystem::new("Root");
        assert!(XtceDocument::add_metadata(
            &mut root,
            ElementKind::TelemetryMetaData
        ));
        XtceDocument::add_collection_item(&mut root, ElementKind::TelemetryParameterTypeSet);
        XtceDocument::add_collection_item(&mut root, ElementKind::TelemetryParameterSet);
        let document = XtceDocument {
            root,
            selection: ElementSelection {
                system_path: Vec::new(),
                kind: ElementKind::SpaceSystem,
            },
            file_name: "test.xml".to_owned(),
        };

        let references = document.references_to(
            &ElementSelection {
                system_path: Vec::new(),
                kind: ElementKind::TelemetryParameterType(0),
            },
            "ParameterType1",
        );

        assert!(
            references
                .iter()
                .any(|reference| reference.contains("Parameter1"))
        );
    }
}
