//! Servidor HTTP — inicialização do listener Axum com graceful shutdown.
//!
//! Escuta em `127.0.0.1:8080` e aguarda o sinal Ctrl+C para encerrar
//! o servidor de forma controlada.

use axum::Router;
use tracing::info;

/// Inicializa o servidor HTTP no endereço `127.0.0.1:8080`.
///
/// Recebe um [`Router`] Axum configurado e inicia o listener com
/// graceful shutdown via sinal Ctrl+C.
///
/// # Errors
///
/// Retorna erro se o bind da porta falhar ou se o servidor encontrar
/// um erro fatal durante a execução.
pub async fn server(app: Router) -> Result<(), anyhow::Error> {
    info!("Server starting on http://localhost:8080");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Falha ao instalar handler de Ctrl+C");
    info!("Sinal de desligamento recebido, encerrando servidor...");
}