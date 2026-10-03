use gpui::SharedString;
use gpui_component::IconNamed;

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum OriginalIcon {
    Github,
    ExternalLink,
    Coffee,
    RefreshCw,
    Check,
    AlertCircle,
    Download,
    X,
    List,
    Settings,
    Terminal,
    TerminalThin,
    Bug,
    Info,
    Book,
    Save,
    Plus,
    Trash,
    ChevronUp,
    ChevronDown,
    Edit,
    AlignLeft,
    Copy,
    Minus,
    Square,
    Maximize2,
    Moon,
    Sun,
}

impl IconNamed for OriginalIcon {
    fn path(self) -> SharedString {
        match self {
            Self::Github => "icons/lucide/github.svg",
            Self::ExternalLink => "icons/lucide/external-link.svg",
            Self::Coffee => "icons/lucide/coffee.svg",
            Self::RefreshCw => "icons/lucide/refresh-cw.svg",
            Self::Check => "icons/lucide/check.svg",
            Self::AlertCircle => "icons/lucide/alert-circle.svg",
            Self::Download => "icons/lucide/download.svg",
            Self::X => "icons/lucide/x.svg",
            Self::List => "icons/lucide/list.svg",
            Self::Settings => "icons/lucide/settings.svg",
            Self::Terminal => "icons/lucide/terminal.svg",
            Self::TerminalThin => "icons/lucide/terminal-thin.svg",
            Self::Bug => "icons/lucide/bug.svg",
            Self::Info => "icons/lucide/info.svg",
            Self::Book => "icons/lucide/book.svg",
            Self::Save => "icons/lucide/save.svg",
            Self::Plus => "icons/lucide/plus.svg",
            Self::Trash => "icons/lucide/trash.svg",
            Self::ChevronUp => "icons/lucide/chevron-up.svg",
            Self::ChevronDown => "icons/lucide/chevron-down.svg",
            Self::Edit => "icons/lucide/edit.svg",
            Self::AlignLeft => "icons/lucide/align-left.svg",
            Self::Copy => "icons/lucide/copy.svg",
            Self::Minus => "icons/lucide/minus.svg",
            Self::Square => "icons/lucide/square.svg",
            Self::Maximize2 => "icons/lucide/maximize-2.svg",
            Self::Moon => "icons/lucide/moon.svg",
            Self::Sun => "icons/lucide/sun.svg",
        }
        .into()
    }
}
