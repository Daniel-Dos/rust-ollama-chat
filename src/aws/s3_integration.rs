//! Upload de respostas do chat para AWS S3.
//!
//! Utiliza o SDK AWS configurado via cadeia de provedores padrão
//! (variáveis de ambiente, perfil, IMDS, etc.) para listar buckets
//! e fazer upload de arquivos `.md` com nomes aleatórios.

use anyhow::Context;
use aws_config::BehaviorVersion;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::primitives::ByteStream;
use rand::RngExt;
use rand::distr::Alphabetic;
use std::ops::Add;
use tracing;
use tracing::info;

async fn configure_aws() -> Result<aws_config::SdkConfig, anyhow::Error> {
    info!("Obtendo a configuração do aws");
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;

    Ok(config)
}

fn client_new(config: &aws_config::SdkConfig) -> S3Client {
    info!("Obtendo o cliente S3");
    S3Client::new(config)
}

/// Obtém o nome do primeiro bucket S3 disponível na conta.
///
/// Lista todos os buckets da conta AWS configurada e retorna o nome
/// do primeiro encontrado.
///
/// # Errors
///
/// Retorna erro se a configuração AWS falhar, se não houver buckets
/// na conta, ou se a listagem falhar.
pub async fn get_my_bucket() -> Result<String, anyhow::Error> {
    let config = configure_aws()
        .await
        .inspect_err(|e| tracing::error!("Falha ao configurar AWS: {e}"))?;
    let client_s3 = client_new(&config);

    let list_buckets_output = client_s3
        .list_buckets()
        .send()
        .await
        .inspect_err(|e| tracing::error!("Falha ao listar buckets: {e}"))?;

    let meu_bucket = list_buckets_output
        .buckets()
        .first()
        .and_then(|bucket| bucket.name().map(|name| name.to_string()))
        .context(anyhow::anyhow!("Nenhum bucket encontrado"))?;

    info!(bucket_name = %meu_bucket,"Bucket S3 selecionado");
    Ok(meu_bucket)
}

/// Faz upload de uma string como arquivo `.md` para o bucket S3 especificado.
///
/// Gera um nome de arquivo aleatório de 10 caracteres e faz upload
/// do conteúdo com a extensão `.md`.
///
/// # Errors
///
/// Retorna erro se a configuração AWS falhar ou se o upload falhar.
pub async fn upload_bucket(my_bucket: &str, payload: String) -> Result<String, anyhow::Error> {
    let config = configure_aws()
        .await
        .inspect_err(|e| tracing::error!("Falha ao configurar AWS: {e}"))?;
    let client_s3 = client_new(&config);
    let file_name= generete_random_file_name().add(".md");

    let upload_s3 = client_s3
        .put_object()
        .bucket(my_bucket)
        .key(&file_name)
        .body(ByteStream::from(payload.into_bytes()))
        .send()
        .await
        .inspect_err(|e| tracing::error!("Falha ao enviar arquivo para S3: {e}"))?;

    info!(
        "Arquivo enviado para o bucket S3: {}",
        upload_s3.e_tag().unwrap_or_default()
    );
    Ok((file_name))
}

fn generete_random_file_name() -> String {
    let rng = rand::rng();
    rng.sample_iter(&Alphabetic)
        .take(10)
        .map(char::from)
        .collect()
}

