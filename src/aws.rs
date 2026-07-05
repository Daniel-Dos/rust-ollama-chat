//! Integração com AWS S3 para upload de respostas do chat.
//!
//! Utiliza o SDK AWS para listar buckets e fazer upload de arquivos `.md`
//! com nomes aleatórios para o primeiro bucket disponível na conta.

pub mod s3_integration;
