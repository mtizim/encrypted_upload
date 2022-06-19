use std::{ffi::OsString, fs::OpenOptions, io::Read, path::Path};

use actix_web::{
    get,
    http::header::{ContentDisposition, DispositionParam, DispositionType},
    web::{self, Bytes},
    HttpResponse, Responder,
};
use serde::Deserialize;

use crate::config::AppConfig;
use chacha20poly1305::{
    aead::{Aead, NewAead},
    Key, XChaCha20Poly1305, XNonce,
};

#[derive(Deserialize)]
pub struct DownloadKey {
    key: String,
}

#[get("/files/{filename}")]
pub async fn download(
    config: web::Data<AppConfig>,
    filename: web::Path<String>,
    key: web::Query<DownloadKey>,
) -> impl Responder {
    let key = &key.key;
    let file_dir = {
        let mut file_dir = config.file_dir.to_owned();
        file_dir.push(filename.as_str());
        file_dir
    };
    let filepath = file_dir.as_path();
    let decoded_file = decode_file(filepath, key).await;

    if decoded_file.is_err() {
        return HttpResponse::NotFound().finish();
    }
    let decoded_file = decoded_file.unwrap();
    dbg!("vend");
    HttpResponse::Ok()
        .content_type("application/octet-stream")
        .append_header(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![DispositionParam::Filename(decoded_file.filename)],
        })
        .body(decoded_file.file)
}

struct DecodedFile {
    filename: String,
    file: Bytes,
}

// I'm discarding error data a lot here - not a good decision, but helps with faster debugging
// when I'm not handling many error cases aside from a generic error message
async fn decode_file(filepath: &Path, key: &String) -> Result<DecodedFile, ()> {
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key.as_bytes()));

    // * Doesn't matter - the keys are single use
    let nonce = XNonce::from_slice(&[0; 24]);
    let nonce_filename = XNonce::from_slice(&[1; 24]);

    let filepath = filepath.to_owned();

    let filepath_st = filepath.to_owned();
    let file_data = web::block(|| {
        let f = OpenOptions::new().read(true).open(filepath_st);
        let mut buffer: Vec<u8> = Vec::new();
        let res = f.map(|mut f| f.read_to_end(&mut buffer));
        res.map(|_| buffer)
    })
    .await
    .or(Err(()))?
    .or(Err(()))?;

    let filename = filepath.file_name().ok_or(())?;
    let filename: OsString = filename.into();
    let filename = filename.into_string().or(Err(()))?;
    dbg!(&filename);
    let dec_filename = String::from_utf8({
        let x = cipher
            .decrypt(nonce_filename, filename.as_ref())
            .or(Err(()));
        dbg!(&x);
        x
    }?)
    .or(Err(()))?;
    dbg!("dec fname");
    let dec_filedata = cipher.decrypt(nonce, file_data.as_ref()).or(Err(()))?;
    dbg!("dec fdata");
    let dec_filedata = Bytes::from_iter(dec_filedata);

    Ok(DecodedFile {
        filename: dec_filename,
        file: dec_filedata,
    })
}
