use std::collections::VecDeque;

use gpui::{
    AnyElement, App, AppContext as _, ClipboardItem, Context, Entity, IntoElement, ParentElement,
    Render, Styled, Subscription, Task, Window, WindowControlArea, div, prelude::*, px, rgb,
};
use gpui_component::{
    Icon, Root, Sizable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement as _,
    v_flex,
};

use crate::{
    STATE,
    config::{Config, OnCopyMode},
    events::{AppEvent, ConversionLog, TraceLog},
    icons::OriginalIcon,
    tsf_availability::check_tsf_availability,
    vr,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Page {
    Home,
    Settings,
    Logs,
    About,
}

impl Page {
    fn label(self) -> &'static str {
        match self {
            Self::Home => "ログ",
            Self::Settings => "設定",
            Self::Logs => "デバッグ",
            Self::About => "情報",
        }
    }

    fn icon(self) -> OriginalIcon {
        match self {
            Self::Home => OriginalIcon::List,
            Self::Settings => OriginalIcon::Settings,
            Self::Logs => OriginalIcon::Bug,
            Self::About => OriginalIcon::Info,
        }
    }
}

pub struct AppView {
    page: Page,
    config: Config,
    conversion_logs: VecDeque<ConversionLog>,
    trace_logs: VecDeque<TraceLog>,
    status: Option<String>,
    dark_mode: bool,
    auto_scroll: bool,

    prefix_input: Entity<InputState>,
    split_input: Entity<InputState>,
    command_input: Entity<InputState>,
    log_filter_input: Entity<InputState>,
    log_filter: String,

    _subscriptions: Vec<Subscription>,
    _event_task: Task<()>,
}

impl AppView {
    pub fn new(
        receiver: flume::Receiver<AppEvent>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let config = STATE.lock().unwrap().clone();
        let prefix_input = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(config.prefix.clone())
                .placeholder("例: ;")
        });
        let split_input = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(config.split.clone())
                .placeholder("例: /")
        });
        let command_input = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(config.command.clone())
                .placeholder("例: ;")
        });
        let log_filter_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("ログをフィルタ..."));

        let subscriptions = vec![
            cx.subscribe(&prefix_input, Self::on_prefix_changed),
            cx.subscribe(&split_input, Self::on_split_changed),
            cx.subscribe(&command_input, Self::on_command_changed),
            cx.subscribe(&log_filter_input, Self::on_log_filter_changed),
        ];

        let event_task = cx.spawn(async move |this, cx| {
            while let Ok(event) = receiver.recv_async().await {
                let Some(this) = this.upgrade() else {
                    break;
                };
                if this
                    .update(cx, |view, cx| {
                        match event {
                            AppEvent::Conversion(log) => {
                                view.conversion_logs.push_front(log);
                                view.conversion_logs.truncate(200);
                            }
                            AppEvent::Trace(log) => {
                                view.trace_logs.push_back(log);
                                while view.trace_logs.len() > 1000 {
                                    view.trace_logs.pop_front();
                                }
                            }
                        }
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        });

        Self {
            page: Page::Home,
            config,
            conversion_logs: VecDeque::new(),
            trace_logs: VecDeque::new(),
            status: None,
            dark_mode: false,
            auto_scroll: true,
            prefix_input,
            split_input,
            command_input,
            log_filter_input,
            log_filter: String::new(),
            _subscriptions: subscriptions,
            _event_task: event_task,
        }
    }

    fn text(&self) -> gpui::Rgba {
        if self.dark_mode {
            rgb(0xe5e7eb)
        } else {
            rgb(0x374151)
        }
    }

    fn muted_text(&self) -> gpui::Rgba {
        if self.dark_mode {
            rgb(0x9ca3af)
        } else {
            rgb(0x6b7280)
        }
    }

    fn border(&self) -> gpui::Rgba {
        if self.dark_mode {
            rgb(0x374151)
        } else {
            rgb(0xf3f4f6)
        }
    }

    fn card(&self) -> gpui::Rgba {
        if self.dark_mode {
            rgb(0x1f2937)
        } else {
            rgb(0xffffff)
        }
    }

    fn soft_background(&self) -> gpui::Rgba {
        if self.dark_mode {
            rgb(0x374151)
        } else {
            rgb(0xf9fafb)
        }
    }

    fn on_prefix_changed(
        &mut self,
        input: Entity<InputState>,
        event: &InputEvent,
        cx: &mut Context<Self>,
    ) {
        if matches!(event, InputEvent::Change) {
            self.config.prefix = input.read(cx).value().to_string();
            self.persist_config(cx);
        }
    }

    fn on_split_changed(
        &mut self,
        input: Entity<InputState>,
        event: &InputEvent,
        cx: &mut Context<Self>,
    ) {
        if matches!(event, InputEvent::Change) {
            self.config.split = input.read(cx).value().to_string();
            self.persist_config(cx);
        }
    }

    fn on_command_changed(
        &mut self,
        input: Entity<InputState>,
        event: &InputEvent,
        cx: &mut Context<Self>,
    ) {
        if matches!(event, InputEvent::Change) {
            self.config.command = input.read(cx).value().to_string();
            self.persist_config(cx);
        }
    }

    fn on_log_filter_changed(
        &mut self,
        input: Entity<InputState>,
        event: &InputEvent,
        cx: &mut Context<Self>,
    ) {
        if matches!(event, InputEvent::Change) {
            self.log_filter = input.read(cx).value().to_string().to_lowercase();
            cx.notify();
        }
    }

    fn persist_config(&mut self, cx: &mut Context<Self>) {
        self.config.force_cpu_backend();
        match self.config.save() {
            Ok(()) => {
                *STATE.lock().unwrap() = self.config.clone();
                self.status = Some("保存完了".into());
            }
            Err(error) => self.status = Some(format!("保存失敗: {error}")),
        }
        cx.notify();
    }

    fn render_title_bar(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let icon_color = rgb(0xffffff);
        let control = |id: &'static str, icon: OriginalIcon, size: f32, area: WindowControlArea| {
            div()
                .id(id)
                .w(px(34.))
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(icon_color)
                .window_control_area(area)
                .hover(|style| style.bg(rgb(0x4338ca)))
                .child(Icon::new(icon).with_size(px(size)))
        };

        h_flex()
            .h(px(32.))
            .flex_shrink_0()
            .bg(if self.dark_mode {
                rgb(0x312e81)
            } else {
                rgb(0x4f46e5)
            })
            .text_color(rgb(0xffffff))
            .child(
                h_flex()
                    .h_full()
                    .flex_1()
                    .px_2()
                    .gap_1()
                    .window_control_area(WindowControlArea::Drag)
                    .child(div().text_xs().font_medium().child("VRClipboard-IME"))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xc7d2fe))
                            .child(format!("v{}", env!("CARGO_PKG_VERSION"))),
                    ),
            )
            .child(
                div()
                    .id("toggle-theme")
                    .w(px(34.))
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(0x4338ca)))
                    .child(
                        Icon::new(if self.dark_mode {
                            OriginalIcon::Sun
                        } else {
                            OriginalIcon::Moon
                        })
                        .with_size(px(12.)),
                    )
                    .on_click(cx.listener(|view, _, window, cx| {
                        view.dark_mode = !view.dark_mode;
                        let mode = if view.dark_mode {
                            gpui_component::ThemeMode::Dark
                        } else {
                            gpui_component::ThemeMode::Light
                        };
                        gpui_component::Theme::change(mode, Some(window), cx);
                        cx.notify();
                    })),
            )
            .child(control(
                "minimize",
                OriginalIcon::Minus,
                12.,
                WindowControlArea::Min,
            ))
            .child(control(
                "maximize",
                if window.is_maximized() {
                    OriginalIcon::Square
                } else {
                    OriginalIcon::Maximize2
                },
                12.,
                WindowControlArea::Max,
            ))
            .child(control(
                "close",
                OriginalIcon::X,
                14.,
                WindowControlArea::Close,
            ))
            .into_any_element()
    }

    fn nav_item(&self, page: Page, cx: &mut Context<Self>) -> AnyElement {
        let selected = self.page == page;
        let id = match page {
            Page::Home => "nav-home",
            Page::Settings => "nav-settings",
            Page::Logs => "nav-logs",
            Page::About => "nav-about",
        };
        h_flex()
            .id(id)
            .w_full()
            .h(px(32.))
            .px_3()
            .gap_2()
            .rounded_md()
            .cursor_pointer()
            .text_sm()
            .bg(if selected {
                if self.dark_mode {
                    rgb(0x312e81)
                } else {
                    rgb(0xe0e7ff)
                }
            } else {
                self.card()
            })
            .text_color(if selected {
                if self.dark_mode {
                    rgb(0xa5b4fc)
                } else {
                    rgb(0x4338ca)
                }
            } else {
                self.muted_text()
            })
            .hover(|style| {
                if selected {
                    style
                } else if self.dark_mode {
                    style.bg(rgb(0x374151))
                } else {
                    style.bg(rgb(0xf3f4f6))
                }
            })
            .child(
                div()
                    .size(px(20.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(Icon::new(page.icon()).with_size(px(16.))),
            )
            .child(page.label())
            .on_click(cx.listener(move |view, _, _, cx| {
                view.page = page;
                view.status = None;
                cx.notify();
            }))
            .into_any_element()
    }

    fn render_sidebar(&self, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .w(px(144.))
            .h_full()
            .flex_shrink_0()
            .py_2()
            .px_1()
            .border_r_1()
            .border_color(self.border())
            .bg(self.card())
            .child(
                v_flex()
                    .flex_1()
                    .gap_1()
                    .child(self.nav_item(Page::Home, cx))
                    .child(self.nav_item(Page::Settings, cx)),
            )
            .child(
                v_flex()
                    .gap_1()
                    .pt_2()
                    .border_t_1()
                    .border_color(self.border())
                    .child(self.nav_item(Page::Logs, cx))
                    .child(self.nav_item(Page::About, cx)),
            )
            .into_any_element()
    }

    fn page_heading(&self, icon: OriginalIcon, title: &str, margin_bottom: f32) -> AnyElement {
        h_flex()
            .gap(px(6.))
            .mb(px(margin_bottom))
            .text_base()
            .font_medium()
            .text_color(self.text())
            .child(Icon::new(icon).with_size(px(16.)))
            .child(title.to_string())
            .into_any_element()
    }

    fn section_heading(&self, title: &str) -> AnyElement {
        div()
            .w_full()
            .pb_1()
            .mb_2()
            .border_b_1()
            .border_color(self.border())
            .text_sm()
            .font_medium()
            .text_color(self.text())
            .child(title.to_string())
            .into_any_element()
    }

    fn input_field(
        &self,
        label: &str,
        description: &str,
        input: &Entity<InputState>,
        disabled: bool,
    ) -> AnyElement {
        v_flex()
            .mb_3()
            .child(
                div()
                    .mb_1()
                    .text_xs()
                    .font_medium()
                    .text_color(self.text())
                    .child(label.to_string()),
            )
            .child(Input::new(input).small().w_full().disabled(disabled))
            .child(
                div()
                    .mt(px(2.))
                    .text_xs()
                    .text_color(self.muted_text())
                    .child(description.to_string()),
            )
            .into_any_element()
    }

    fn checkbox_indicator(&self, checked: bool, disabled: bool) -> AnyElement {
        div()
            .size(px(14.))
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_center()
            .rounded_sm()
            .border_1()
            .border_color(if checked {
                rgb(0x4f46e5)
            } else if self.dark_mode {
                rgb(0x4b5563)
            } else {
                rgb(0xd1d5db)
            })
            .bg(if checked {
                if disabled {
                    rgb(0xa5b4fc)
                } else {
                    rgb(0x4f46e5)
                }
            } else {
                self.card()
            })
            .text_color(rgb(0xffffff))
            .when(checked, |this| {
                this.child(Icon::new(OriginalIcon::Check).with_size(px(10.)))
            })
            .into_any_element()
    }

    fn render_home(&self) -> AnyElement {
        let log_area: AnyElement = if self.conversion_logs.is_empty() {
            v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .text_color(if self.dark_mode {
                    rgb(0x6b7280)
                } else {
                    rgb(0x9ca3af)
                })
                .child(Icon::new(OriginalIcon::TerminalThin).with_size(px(24.)))
                .child(div().mt_2().text_sm().child("ログはまだありません"))
                .child(
                    div()
                        .text_xs()
                        .child("テキストを変換すると、ここに表示されます"),
                )
                .into_any_element()
        } else {
            v_flex()
                .gap_2()
                .children(self.conversion_logs.iter().map(|log| {
                    v_flex()
                        .p_2()
                        .rounded_md()
                        .border_1()
                        .border_color(self.border())
                        .bg(self.card())
                        .child(
                            div()
                                .mb_1()
                                .text_xs()
                                .text_color(self.muted_text())
                                .child(log.time.clone()),
                        )
                        .child(
                            h_flex()
                                .gap_2()
                                .text_sm()
                                .child(
                                    div()
                                        .px_2()
                                        .py_1()
                                        .rounded_sm()
                                        .bg(self.soft_background())
                                        .child(log.original.clone()),
                                )
                                .child(div().text_xs().text_color(rgb(0x9ca3af)).child("→"))
                                .child(
                                    div()
                                        .px_2()
                                        .py_1()
                                        .rounded_sm()
                                        .bg(if self.dark_mode {
                                            rgb(0x064e3b)
                                        } else {
                                            rgb(0xecfdf5)
                                        })
                                        .text_color(if self.dark_mode {
                                            rgb(0x6ee7b7)
                                        } else {
                                            rgb(0x059669)
                                        })
                                        .child(log.converted.clone()),
                                ),
                        )
                }))
                .into_any_element()
        };

        v_flex()
            .size_full()
            .child(self.page_heading(OriginalIcon::Terminal, "変換ログ", 8.))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .p_2()
                    .overflow_y_scrollbar()
                    .rounded_md()
                    .border_1()
                    .border_color(self.border())
                    .bg(self.soft_background())
                    .child(log_area),
            )
            .into_any_element()
    }

    fn render_settings(&self, cx: &mut Context<Self>) -> AnyElement {
        let conversion_locked = self.config.use_tsf_reconvert || self.config.use_azookey_conversion;
        let current_mode = match self.config.on_copy_mode {
            OnCopyMode::ReturnToClipboard => "クリップボードへ送信",
            OnCopyMode::ReturnToChatbox => "チャットボックスへ送信",
            OnCopyMode::SendDirectly => "直接チャットへ送信",
        };

        let basic = v_flex()
            .flex_1()
            .min_w_0()
            .child(self.section_heading("基本設定"))
            .child(self.input_field(
                "区切り文字",
                "複数の変換モードを使いたい場合の区切り文字",
                &self.split_input,
                conversion_locked,
            ))
            .child(self.input_field(
                "モード変更文字",
                "変換モードを変更するための文字",
                &self.command_input,
                conversion_locked,
            ))
            .child(
                h_flex()
                    .id("ignore-prefix")
                    .mb_2()
                    .gap_2()
                    .text_xs()
                    .text_color(if conversion_locked {
                        self.muted_text()
                    } else {
                        self.text()
                    })
                    .when(!conversion_locked, |this| {
                        this.cursor_pointer()
                            .on_click(cx.listener(|view, _, _, cx| {
                                view.config.ignore_prefix = !view.config.ignore_prefix;
                                view.persist_config(cx);
                            }))
                    })
                    .child(self.checkbox_indicator(self.config.ignore_prefix, conversion_locked))
                    .child("無条件で変換"),
            )
            .child(self.input_field(
                "開始文字",
                "変換を開始する文字（無条件で変換がオンの場合は無効）",
                &self.prefix_input,
                self.config.ignore_prefix || conversion_locked,
            ))
            .child(
                v_flex()
                    .mb_3()
                    .child(
                        div()
                            .mb_1()
                            .text_xs()
                            .font_medium()
                            .text_color(self.text())
                            .child("コピー時の動作"),
                    )
                    .child(
                        h_flex()
                            .id("copy-mode")
                            .w_full()
                            .h(px(30.))
                            .px_2()
                            .justify_between()
                            .rounded_md()
                            .border_1()
                            .border_color(if self.dark_mode {
                                rgb(0x4b5563)
                            } else {
                                rgb(0xd1d5db)
                            })
                            .bg(self.card())
                            .text_sm()
                            .cursor_pointer()
                            .hover(|style| style.border_color(rgb(0x818cf8)))
                            .child(current_mode)
                            .child(Icon::new(OriginalIcon::ChevronDown).with_size(px(14.)))
                            .on_click(cx.listener(|view, _, _, cx| {
                                view.config.on_copy_mode = match view.config.on_copy_mode {
                                    OnCopyMode::ReturnToClipboard => OnCopyMode::ReturnToChatbox,
                                    OnCopyMode::ReturnToChatbox => OnCopyMode::SendDirectly,
                                    OnCopyMode::SendDirectly => OnCopyMode::ReturnToClipboard,
                                };
                                view.persist_config(cx);
                            })),
                    ),
            );

        let advanced_box = v_flex()
            .mt_2()
            .p_2()
            .rounded_md()
            .border_1()
            .border_color(if self.dark_mode { rgb(0x4b5563) } else { rgb(0xe5e7eb) })
            .bg(self.soft_background())
            .child(
                div()
                    .pb_1()
                    .mb_2()
                    .text_xs()
                    .font_medium()
                    .text_color(self.text())
                    .child("高度な変換方式"),
            )
            .child(
                h_flex()
                    .id("azookey")
                    .gap_2()
                    .text_xs()
                    .cursor_pointer()
                    .child(self.checkbox_indicator(self.config.use_azookey_conversion, false))
                    .child("AzooKey変換")
                    .on_click(cx.listener(|view, _, _, cx| {
                        view.config.use_azookey_conversion = !view.config.use_azookey_conversion;
                        if view.config.use_azookey_conversion {
                            view.config.use_tsf_reconvert = false;
                        }
                        view.persist_config(cx);
                    })),
            )
            .child(
                div()
                    .ml_5()
                    .mt(px(2.))
                    .text_xs()
                    .text_color(self.muted_text())
                    .child("azooKey変換機能を使用します。有効にすると基本入力が無効化されます。"),
            )
            .child(
                h_flex()
                    .id("tsf")
                    .mt_3()
                    .gap_2()
                    .text_xs()
                    .cursor_pointer()
                    .child(self.checkbox_indicator(self.config.use_tsf_reconvert, false))
                    .child("TSF再変換")
                    .on_click(cx.listener(|view, _, _, cx| {
                            if view.config.use_tsf_reconvert {
                                view.config.use_tsf_reconvert = false;
                                view.persist_config(cx);
                                return;
                            }
                            match check_tsf_availability() {
                                Ok(true) => {
                                    view.config.use_tsf_reconvert = true;
                                    view.config.use_azookey_conversion = false;
                                    view.persist_config(cx);
                                }
                                Ok(false) => {
                                    view.status = Some("TSF再変換を利用できません。Microsoft IMEの互換性設定を確認してください".into());
                                    cx.notify();
                                }
                                Err(error) => {
                                    view.status = Some(format!("TSF確認に失敗しました: {error}"));
                                    cx.notify();
                                }
                            }
                        })),
            )
            .child(
                div()
                    .ml_5()
                    .mt(px(2.))
                    .text_xs()
                    .text_color(self.muted_text())
                    .child("非推奨です。以前のバージョンの Microsoft IME が必要です。"),
            );

        let steam_box = h_flex()
            .mt_4()
            .p_2()
            .justify_between()
            .rounded_md()
            .border_1()
            .border_color(if self.dark_mode {
                rgb(0x4b5563)
            } else {
                rgb(0xe5e7eb)
            })
            .bg(self.soft_background())
            .child(
                v_flex()
                    .child(div().text_xs().font_medium().child("SteamVR連携"))
                    .child(
                        div()
                            .mt_1()
                            .text_xs()
                            .text_color(self.muted_text())
                            .child("オーバーレイアプリケーションとして登録します"),
                    ),
            )
            .child(
                div()
                    .id("register-manifest")
                    .px_3()
                    .py_1()
                    .rounded_md()
                    .bg(rgb(0x6366f1))
                    .text_color(rgb(0xffffff))
                    .text_xs()
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(0x4f46e5)))
                    .child("登録")
                    .on_click(cx.listener(|view, _, _, cx| {
                        view.status = Some(
                            match vr::create_vrmanifest()
                                .and_then(|_| vr::register_manifest_with_openvr())
                            {
                                Ok(()) => "登録完了".into(),
                                Err(error) => format!("登録失敗: {error}"),
                            },
                        );
                        cx.notify();
                    })),
            );

        let advanced = v_flex()
            .flex_1()
            .min_w_0()
            .child(self.section_heading("詳細設定"))
            .child(
                h_flex()
                    .id("skip-url")
                    .mb_2()
                    .gap_2()
                    .text_xs()
                    .cursor_pointer()
                    .child(self.checkbox_indicator(self.config.skip_url, false))
                    .child("URL が含まれている文章をスキップ")
                    .on_click(cx.listener(|view, _, _, cx| {
                        view.config.skip_url = !view.config.skip_url;
                        view.persist_config(cx);
                    })),
            )
            .child(
                h_flex()
                    .id("skip-outside-vrc")
                    .mb_2()
                    .gap_2()
                    .text_xs()
                    .cursor_pointer()
                    .child(self.checkbox_indicator(self.config.skip_on_out_of_vrc, false))
                    .child("VRChat以外からのコピーをスキップ")
                    .on_click(cx.listener(|view, _, _, cx| {
                        view.config.skip_on_out_of_vrc = !view.config.skip_on_out_of_vrc;
                        view.persist_config(cx);
                    })),
            )
            .child(advanced_box)
            .child(steam_box);

        v_flex()
            .size_full()
            .child(
                h_flex()
                    .justify_between()
                    .mb_2()
                    .child(
                        h_flex()
                            .gap_1()
                            .text_base()
                            .font_medium()
                            .text_color(self.text())
                            .child(Icon::new(OriginalIcon::Settings).with_size(px(16.)))
                            .child("設定"),
                    )
                    .when_some(self.status.clone(), |this, status| {
                        this.child(
                            div()
                                .text_xs()
                                .text_color(if status.contains("失敗") {
                                    rgb(0xef4444)
                                } else {
                                    rgb(0x22c55e)
                                })
                                .child(status),
                        )
                    }),
            )
            .child(
                div()
                    .w_full()
                    .p_3()
                    .rounded_md()
                    .border_1()
                    .border_color(self.border())
                    .bg(self.card())
                    .child(h_flex().items_start().gap_4().child(basic).child(advanced)),
            )
            .into_any_element()
    }

    fn render_logs(&self, cx: &mut Context<Self>) -> AnyElement {
        let filtered = self.trace_logs.iter().filter(|log| {
            self.log_filter.is_empty()
                || log.message.to_lowercase().contains(&self.log_filter)
                || log.level.to_lowercase().contains(&self.log_filter)
                || log.module_path.to_lowercase().contains(&self.log_filter)
        });

        let terminal = v_flex()
            .flex_1()
            .min_h_0()
            .gap_1()
            .p_2()
            .overflow_y_scrollbar()
            .rounded_md()
            .border_1()
            .border_color(rgb(0x374151))
            .bg(rgb(0x000000))
            .font_family("Consolas")
            .text_xs()
            .children(filtered.map(|log| {
                let level_color = match log.level.as_str() {
                    "ERROR" => rgb(0xef4444),
                    "WARN" => rgb(0xca8a04),
                    "INFO" => rgb(0x3b82f6),
                    "DEBUG" => rgb(0xa855f7),
                    _ => rgb(0x6b7280),
                };
                div().text_color(level_color).child(format!(
                    "[{} {}] [{}] {}",
                    log.timestamp, log.module_path, log.level, log.message
                ))
            }));

        v_flex()
            .size_full()
            .child(
                h_flex()
                    .justify_between()
                    .mb_2()
                    .child(div().text_base().font_medium().child("デバッグログ"))
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("copy-logs")
                                    .ghost()
                                    .xsmall()
                                    .icon(OriginalIcon::Copy)
                                    .label("コピー")
                                    .on_click(cx.listener(|view, _, _, cx| {
                                        cx.write_to_clipboard(ClipboardItem::new_string(
                                            view.logs_as_text(),
                                        ));
                                        view.status = Some("コピー完了".into());
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("clear-logs")
                                    .ghost()
                                    .xsmall()
                                    .icon(OriginalIcon::Trash)
                                    .label("クリア")
                                    .on_click(cx.listener(|view, _, _, cx| {
                                        view.trace_logs.clear();
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("save-logs")
                                    .ghost()
                                    .xsmall()
                                    .icon(OriginalIcon::Download)
                                    .label("保存")
                                    .on_click(cx.listener(|view, _, _, cx| {
                                        let path =
                                            Config::get_path().join("vrclipboard-ime-debug.log");
                                        view.status = Some(
                                            match std::fs::write(&path, view.logs_as_text()) {
                                                Ok(()) => "保存完了".into(),
                                                Err(error) => format!("保存失敗: {error}"),
                                            },
                                        );
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            .child(
                h_flex()
                    .mb_2()
                    .gap_2()
                    .child(
                        div()
                            .flex_1()
                            .child(Input::new(&self.log_filter_input).small().w_full()),
                    )
                    .child(
                        h_flex()
                            .id("auto-scroll")
                            .gap_1()
                            .text_xs()
                            .cursor_pointer()
                            .child(self.checkbox_indicator(self.auto_scroll, false))
                            .child("自動スクロール")
                            .on_click(cx.listener(|view, _, _, cx| {
                                view.auto_scroll = !view.auto_scroll;
                                cx.notify();
                            })),
                    ),
            )
            .child(terminal)
            .into_any_element()
    }

    fn logs_as_text(&self) -> String {
        self.trace_logs
            .iter()
            .map(|log| {
                format!(
                    "[{} {}] [{}] {}",
                    log.timestamp, log.module_path, log.level, log.message
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn info_row(&self, label: &str, value: &str) -> AnyElement {
        h_flex()
            .mb_1()
            .text_sm()
            .child(
                div()
                    .w(px(80.))
                    .text_color(self.text())
                    .child(label.to_string()),
            )
            .child(div().text_color(self.muted_text()).child(value.to_string()))
            .into_any_element()
    }

    fn render_about(&self, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .size_full()
            .child(self.page_heading(OriginalIcon::Coffee, "アプリケーション情報", 16.))
            .child(
                v_flex()
                    .w_full()
                    .p_4()
                    .rounded_md()
                    .border_1()
                    .border_color(self.border())
                    .bg(self.card())
                    .child(
                        h_flex()
                            .mb_4()
                            .gap_3()
                            .child(
                                div()
                                    .text_lg()
                                    .font_semibold()
                                    .text_color(rgb(0x4f46e5))
                                    .child("VRClipboard-IME"),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .rounded_md()
                                    .bg(if self.dark_mode {
                                        rgb(0x312e81)
                                    } else {
                                        rgb(0xe0e7ff)
                                    })
                                    .text_xs()
                                    .text_color(if self.dark_mode {
                                        rgb(0xa5b4fc)
                                    } else {
                                        rgb(0x4338ca)
                                    })
                                    .child(format!("v{}", env!("CARGO_PKG_VERSION"))),
                            ),
                    )
                    .child(self.section_heading("アプリケーション情報"))
                    .child(self.info_row("バージョン:", env!("CARGO_PKG_VERSION")))
                    .child(self.info_row("ライセンス:", "MIT"))
                    .child(self.info_row("技術:", "GPUI, Rust"))
                    .child(div().h(px(16.)))
                    .child(self.section_heading("開発者"))
                    .child(self.info_row("作者:", "mii443"))
                    .child(self.info_row("VRChat:", "みー mii"))
                    .child(
                        h_flex()
                            .mb_1()
                            .text_sm()
                            .child(div().w(px(80.)).child("GitHub:"))
                            .child(
                                h_flex()
                                    .id("author-github")
                                    .gap_1()
                                    .cursor_pointer()
                                    .text_color(rgb(0x4f46e5))
                                    .child("mii443")
                                    .child(Icon::new(OriginalIcon::ExternalLink).with_size(px(12.)))
                                    .on_click(|_, _, cx| cx.open_url("https://github.com/mii443")),
                            ),
                    )
                    .child(div().h(px(16.)))
                    .child(self.section_heading("リンク"))
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("repository")
                                    .small()
                                    .icon(OriginalIcon::Github)
                                    .label("GitHubリポジトリ")
                                    .on_click(|_, _, cx| {
                                        cx.open_url("https://github.com/mii443/vrclipboard-ime-gui")
                                    }),
                            )
                            .child(
                                Button::new("website")
                                    .small()
                                    .icon(OriginalIcon::ExternalLink)
                                    .label("ウェブサイト")
                                    .on_click(|_, _, cx| cx.open_url("https://vrime.mii.dev")),
                            )
                            .child(
                                Button::new("updates")
                                    .small()
                                    .icon(OriginalIcon::RefreshCw)
                                    .label("更新を確認")
                                    .on_click(cx.listener(|view, _, _, cx| {
                                        view.status = Some("GPUI版の自動更新は準備中です".into());
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            .into_any_element()
    }
}

impl Render for AppView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = match self.page {
            Page::Home => self.render_home(),
            Page::Settings => self.render_settings(cx),
            Page::Logs => self.render_logs(cx),
            Page::About => self.render_about(cx),
        };

        v_flex()
            .size_full()
            .bg(if self.dark_mode {
                rgb(0x111827)
            } else {
                rgb(0xffffff)
            })
            .text_color(self.text())
            .child(self.render_title_bar(window, cx))
            .child(
                h_flex()
                    .flex_1()
                    .min_h_0()
                    .child(self.render_sidebar(cx))
                    .child(
                        div()
                            .flex_1()
                            .size_full()
                            .min_w_0()
                            .p_3()
                            .overflow_y_scrollbar()
                            .child(content),
                    ),
            )
    }
}

pub fn root_view(
    receiver: flume::Receiver<AppEvent>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<Root> {
    let view = cx.new(|cx| AppView::new(receiver, window, cx));
    cx.new(|cx| Root::new(view, window, cx))
}
