use std::{
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};

use actix_multipart::Multipart;
use actix_web::{
    post,
    web::{self, BytesMut},
    HttpResponse, Responder,
};
use chacha20poly1305::Key;
use chacha20poly1305::{
    aead::{Aead, NewAead},
    XChaCha20Poly1305, XNonce,
};
use serde::Serialize;

use futures_util::TryStreamExt as _;
use uuid::Uuid;

use crate::config::AppConfig;
// ! The key could be used twice
// ! The file is limited to be in-memory
// ! The error code should be more descriptive
// ! A filename collision is possible and could be avoided
#[post("/files")]
pub async fn upload(config: web::Data<AppConfig>, payload: Multipart) -> impl Responder {
    let file_dir = &config.file_dir;
    // could be more secure ofc, but isn't because constraints
    let key = Uuid::new_v4().to_string().replace('-', "");
    let save_res = save_file(file_dir, payload, &key).await;

    match save_res {
        Ok(filename) => HttpResponse::Ok().json(web::Json(UploadResponse { key, filename })),
        Err(_) => HttpResponse::ServiceUnavailable().finish(),
    }
}

#[derive(Serialize)]
struct UploadResponse {
    key: String,
    filename: String,
}

async fn save_file(file_dir: &Path, mut payload: Multipart, key: &String) -> Result<String, ()> {
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key.as_bytes()));

    // * Doesn't matter - the keys are single use
    let nonce = XNonce::from_slice(&[0; 24]);
    let nonce_filename = XNonce::from_slice(&[1; 24]);

    while let Some(mut field) = match payload.try_next().await {
        Ok(it) => it,
        Err(_) => return Err(()),
    } {
        if field.name() != "file" {
            continue;
        }
        let content_disposition = field.content_disposition();

        let filename = content_disposition
            .get_filename()
            .unwrap_or("file")
            .to_owned();

        let mut file = BytesMut::new();
        while let Some(chunk) = match field.try_next().await {
            Ok(it) => it,
            Err(_) => return Err(()),
        } {
            file.extend(chunk);
        }

        let enc_file: Vec<u8> = cipher.encrypt(nonce, file.as_ref()).or(Err(()))?;
        let enc_filename: Vec<u8> = cipher
            .encrypt(nonce_filename, filename.as_ref())
            .or(Err(()))?;
        let enc_filename = base64::encode(enc_filename);

        let mut filepath = PathBuf::from(file_dir);
        filepath.push(&enc_filename);

        let mut f = web::block(|| {
            OpenOptions::new()
                .create_new(true)
                .write(true)
                .append(true)
                .open(filepath)
        })
        .await
        .or(Err(()))?
        .or(Err(()))?;

        web::block(move || f.write_all(&enc_file).map(|_| f))
            .await
            .or(Err(()))?
            .or(Err(()))?;

        return Ok(enc_filename);
    }

    Err(())
}
