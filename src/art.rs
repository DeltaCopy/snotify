use image::ImageError;
use image::ImageResult;
use isahc::AsyncReadResponseExt;
use isahc::{config::RedirectPolicy, prelude::*, HttpClient};
use std::time::Duration;
const THUMBNAIL_SIZE: u32 = 256;
const TIMEOUT: Duration = Duration::from_secs(5);
use image::io::Reader as ImageReader;
use std::io::Cursor;
pub struct Art {
    filename: String,
    url: String,
}

impl Art {
    pub fn new(filename: &String, url: &String) -> Self {
        Art {
            filename: filename.to_string(),
            url: url.to_string(),
        }
    }

    // create a new http client instance
    pub fn build_http_client() -> Result<HttpClient, isahc::Error> {
        let http_client = HttpClient::builder()
            .default_headers(&[(
                "User-Agent",
                "Mozilla/5.0 (X11; Linux x86_64; rv:104.0) Gecko/20100101 Firefox/104.0",
            )])
            .timeout(TIMEOUT)
            .redirect_policy(RedirectPolicy::Follow)
            .build()?;

        Ok(http_client)
    }

    pub async fn get_album_art(&self, http_client: &HttpClient) -> Result<Vec<u8>, isahc::Error> {
        let mut img_buffer = vec![];
        http_client
            .get_async(&self.url)
            .await?
            .copy_to(&mut img_buffer)
            .await?;

        Ok(img_buffer)
    }

    // save album art to disk

    // save album art to disk
    //pub fn save_album_art(self, bytes: &Vec<u8>) -> Result<ImageResult<()>,ImageError>{
    pub fn save_album_art(&self, bytes: &Vec<u8>) -> Result<ImageResult<()>, ImageError> {
        let image = image::load_from_memory(&bytes);

        match image {
            Ok(_) => &image,
            Err(e) => panic!("{}", e),
        };

        let reader = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .unwrap();

        let img_thumbnail = &image.unwrap().resize(
            THUMBNAIL_SIZE,
            THUMBNAIL_SIZE,
            image::imageops::FilterType::Nearest,
        );

        let img_save = img_thumbnail.save_with_format(&self.filename, reader.format().unwrap());

        match img_save {
            Ok(_) => Ok(img_save),
            Err(e) => Err(e),
        }
    }
}
