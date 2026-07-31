//! Estrutura de dados para requisição e resposta do endpoint `/api/prompt`.

use serde::{Deserialize, Serialize};

/// Mensagem de requisição/resposta da API REST.
///
/// Usada tanto como query parameter (`GET /api/prompt?texto=<consulta>`)
/// quanto como corpo da resposta JSON contendo o resultado gerado.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageResponse {
    /// Texto do prompt (entrada) ou resposta gerada (saída).
    pub part_text: String,
    pub bucket_name: String,
    pub file_name: String,
}