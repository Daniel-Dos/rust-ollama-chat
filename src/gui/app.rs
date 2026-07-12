//! Aplicação GUI construída com o framework [Iced](https://iced.rs).
//!
//! # Fluxo
//!
//! 1. **Splash**: exibe banner ASCII com barra de progresso animada por 5 segundos.
//! 2. **Chat**: campo de texto, toggle "Buscar na web", botão "Enviar".
//!    - Durante o processamento, exibe "Processando..." em amarelo.
//!    - A resposta é renderizada como Markdown com botão "Copiar".
//!    - Rodapé mostra total de tokens com percentuais de entrada/saída.
//! 3. **Resultado**: devolve a resposta para `main.rs` quando a janela é fechada.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use iced::widget::{
    button, column, container, markdown, row, scrollable, text, text_input, toggler,
};

use iced::{Font, Length, Size, Subscription, Task, alignment, time};
use rmcp::{Peer, RoleClient};

use crate::mcp::web_search_mcp::McpClientManager;
use crate::rig::client_ollama;

/// Estado principal da aplicação GUI.
pub struct App {
    peer: Peer<RoleClient>,
    prompt: String,
    resposta: String,
    parsed_items: Vec<markdown::Item>,
    tokens_input: u64,
    tokens_output: u64,
    tokens_total: u64,
    fase: Fase,
    tick: u64,
    search_web: bool,
    result: Arc<Mutex<Option<String>>>,
}

/// Máquina de estados da interface.
enum Fase {
    /// Tela de splash inicial (5 segundos).
    Splash,
    /// Tela de chat pronta para interação.
    Pronto,
    /// Aguardando resposta do modelo.
    Processando,
    /// Reservado para uso futuro.
    #[expect(dead_code, reason = "reservado para uso futuro")]
    Concluido,
}

/// Eventos manipulados pela aplicação Iced.
#[derive(Debug, Clone)]
enum Message {
    /// Splash terminou — transiciona para `Pronto`.
    SplashPronto,
    /// Usuário digitou no campo de texto.
    PromptAlterado(String),
    /// Usuário alternou o toggle de busca web.
    ToggleSearch(bool),
    /// Usuário clicou em "Enviar".
    Enviar,
    /// Resposta recebida do modelo (sucesso ou erro).
    RespostaRecebida(Result<client_ollama::ChatResult, String>),
    /// Tick do timer de animação do splash.
    Tick,
    /// Placeholder — clique em links será tratado futuramente.
    #[expect(
        dead_code,
        reason = "placeholder - clique em links será tratado futuramente"
    )]
    LinkClicked(markdown::Url),
    /// Usuário clicou em "Copiar".
    Copiar,
}

impl App {
    fn new(peer: Peer<RoleClient>, result: Arc<Mutex<Option<String>>>) -> (Self, Task<Message>) {
        let app = App {
            peer,
            prompt: String::new(),
            resposta: String::new(),
            parsed_items: Vec::new(),
            tokens_input: 0,
            tokens_output: 0,
            tokens_total: 0,
            fase: Fase::Splash,
            tick: 0,
            search_web: true,
            result,
        };
        let task = Task::perform(tokio::time::sleep(Duration::from_secs(5)), |_| {
            Message::SplashPronto
        });
        (app, task)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SplashPronto => {
                self.fase = Fase::Pronto;
                Task::none()
            }
            Message::PromptAlterado(prompt) => {
                self.prompt = prompt;
                Task::none()
            }
            Message::ToggleSearch(v) => {
                self.search_web = v;
                Task::none()
            }
            Message::Enviar => {
                if !matches!(self.fase, Fase::Pronto) {
                    return Task::none();
                }
                let prompt = self.prompt.clone();
                let peer = self.peer.clone();
                let search = self.search_web;
                self.fase = Fase::Processando;

                if search {
                    Task::perform(
                        async move {
                            client_ollama::resposta_chat_peer(peer, prompt)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        Message::RespostaRecebida,
                    )
                } else {
                    Task::perform(
                        async move {
                            client_ollama::chat_direct(prompt)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        Message::RespostaRecebida,
                    )
                }
            }
            Message::RespostaRecebida(Ok(chat)) => {
                self.resposta = chat.resposta;
                self.tokens_input = chat.tokens_input;
                self.tokens_output = chat.tokens_output;
                self.tokens_total = chat.tokens_total;
                self.parsed_items = markdown::parse(&self.resposta).collect();
                self.fase = Fase::Pronto;
                *self.result.lock().unwrap() = Some(self.resposta.clone());
                Task::none()
            }
            Message::RespostaRecebida(Err(e)) => {
                self.resposta = format!("Erro: {e}");
                self.parsed_items = markdown::parse(&self.resposta).collect();
                self.fase = Fase::Pronto;
                *self.result.lock().unwrap() = Some(self.resposta.clone());
                Task::none()
            }
            Message::Tick => {
                self.tick += 1;
                Task::none()
            }
            Message::LinkClicked(_) => Task::none(),
            Message::Copiar => iced::clipboard::write::<Message>(self.resposta.clone()),
        }
    }

    fn view(&self) -> iced::Element<'_, Message> {
        match self.fase {
            Fase::Splash => self.tela_splash(),
            _ => self.tela_chat(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        match self.fase {
            Fase::Splash => time::every(Duration::from_millis(200)).map(|_| Message::Tick),
            _ => Subscription::none(),
        }
    }

    fn tela_splash(&self) -> iced::Element<'_, Message> {
        const BANNER: &str = include_str!("../../banner.txt");
        let banner = text(BANNER)
            .size(16)
            .font(Font::MONOSPACE)
            .color([0.0, 1.0, 1.0]);
        let version = text("Rust Rig AI v0.1.0 — Daniel Dias").size(14);

        let progress = (self.tick as f32) / 25.0;
        let filled = (progress * 20.0).min(20.0) as usize;
        let bar: String = "█".repeat(filled) + &"░".repeat(20 - filled);
        let loading = text(format!("Inicializando  {}  {:.0}%", bar, progress * 100.0))
            .size(14)
            .font(Font::MONOSPACE)
            .color([0.0, 1.0, 0.0]);

        container(
            column![banner, version, loading]
                .align_x(alignment::Horizontal::Center)
                .spacing(10),
        )
        .center(Length::Fill)
        .into()
    }

    fn tela_chat(&self) -> iced::Element<'_, Message> {
        let toggle = toggler(self.search_web)
            .label("Buscar na web")
            .on_toggle(Message::ToggleSearch);

        let mut input = text_input("Digite seu prompt...", &self.prompt);
        if matches!(self.fase, Fase::Pronto) {
            input = input
                .on_input(Message::PromptAlterado)
                .on_submit(Message::Enviar);
        }

        let enviar_btn = {
            let b = button("Enviar");
            if matches!(self.fase, Fase::Pronto) {
                b.on_press(Message::Enviar)
            } else {
                b
            }
        };

        let entrada = row![input, enviar_btn].spacing(10);

        let mut col = column![toggle, entrada]
            .spacing(10)
            .padding(10)
            .height(Length::Fill);

        match self.fase {
            Fase::Processando => {
                col = col.push(text("Processando...").size(14).color([1.0, 1.0, 0.0]));
            }
            _ => {
                if !self.parsed_items.is_empty() {
                    let md_view = markdown::view(
                        &self.parsed_items,
                        markdown::Settings::default(),
                        markdown::Style::from_palette(iced::Theme::Nightfly.palette()),
                    )
                    .map(Message::LinkClicked);

                    col = col.push(
                        row![
                            scrollable(md_view).width(Length::Fill).height(Length::Fill),
                            button("Copiar").on_press(Message::Copiar),
                        ]
                        .spacing(10)
                        .height(Length::Fill),
                    );

                    let total = self.tokens_total;
                    let pct_in = if total > 0 {
                        self.tokens_input as f64 / total as f64 * 100.0
                    } else {
                        0.0
                    };
                    let pct_out = if total > 0 {
                        self.tokens_output as f64 / total as f64 * 100.0
                    } else {
                        0.0
                    };
                    let info = text(format!(
                        "Tokens: {} total — entrada: {} ({:.1}%)  saída: {} ({:.1}%)",
                        total, self.tokens_input, pct_in, self.tokens_output, pct_out,
                    ))
                    .size(12);
                    col = col.push(info);
                }
            }
        }

        col.into()
    }
}

/// Inicializa e executa a janela GUI Iced.
///
/// Recebe o [`McpClientManager`] já inicializado, extrai o peer MCP,
/// abre a janela e bloqueia até o usuário fechá-la.
///
/// # Retorno
///
/// Retorna a última resposta do chat como `String`, ou um erro se a
/// janela for fechada antes de qualquer interação.
///
/// # Errors
///
/// Retorna erro se a janela não puder ser aberta ou se for fechada
/// sem que o usuário tenha enviado um prompt.
pub fn run(mcp: &McpClientManager) -> Result<String, anyhow::Error> {
    let peer = mcp.peer();
    let result = Arc::new(Mutex::new(None));
    let r2 = result.clone();

    iced::application("Rust Rig AI", App::update, App::view)
        .theme(|_| iced::theme::Theme::Nightfly)
        .subscription(App::subscription)
        .window_size(Size::new(800.0, 600.0))
        .run_with(move || App::new(peer, r2))
        .map_err(|e| anyhow::anyhow!("GUI error: {e}"))?;

    result
        .lock()
        .unwrap()
        .take()
        .ok_or_else(|| anyhow::anyhow!("Janela fechada antes de obter resposta."))
}
