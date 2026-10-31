use image::{self, DynamicImage, imageops};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

#[derive(Debug, Clone)]
pub struct Image {
    inner: image::RgbaImage
}

#[derive(Debug, Clone)]
pub struct Tile<T> {
    pub coords: (u32, u32),
    pub image: T
}

impl Image {  

    pub fn inner(&self) -> &image::RgbaImage {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut image::RgbaImage {
        &mut self.inner
    }

    pub fn load(path: &str) -> Self {
        Self { inner: image::open(path).unwrap().to_rgba8() }
    }

    pub fn save(self, path: &str) {
        let di: DynamicImage = self.inner.into();
        di.save(path).unwrap()
    }

    pub fn resize(self, w: u32, h: u32, filter:imageops::FilterType) -> Image {
        let dyn_img: DynamicImage = self.inner.into();
        let result = dyn_img.resize(w, h, filter);
        Image { inner: result.into_rgba8() }
    }

    pub fn from_jpeg_bytes(bytes: Vec<u8>) -> Self {
        let inner = image::load_from_memory_with_format(&bytes, image::ImageFormat::Jpeg)
            .unwrap_or_else(|e| panic!("JPEG decode failed: {e:?}"))
            .into_rgba8();
        Image { inner }
    }

    pub fn into_jpeg_bytes(self, quality: u8) -> Vec<u8> {
        let mut bytes = Vec::new();
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, quality);
        let dyn_image: DynamicImage = self.inner.clone().into();
        dyn_image.write_with_encoder(encoder).unwrap_or_else(|e| panic!("JPEG encode failed: {e:?}"));
        bytes
    }
    
    pub fn into_tiles(self, tile_width: u32, tile_height: u32) -> Vec<Tile<Image>> {
        let mut tiles = Vec::new();

        let width = self.width();
        let height = self.height();

        let dyn_img: DynamicImage = self.inner.into();

        let mut tile_x = 0;
        let mut tile_y = 0;
        for y in (0..height).step_by(tile_height as usize) {
            for x in (0..width).step_by(tile_width as usize) {
                let w = tile_width.min(width - x);
                let h = tile_height.min(height - y);

                let tile = dyn_img.crop_imm(x, y, w, h);
                tiles.push(((tile_x, tile_y), tile));

                tile_x += 1;
            }
            tile_x = 0;
            tile_y += 1;
        }

        tiles.into_iter()
            .map(|(coords, tile)| Tile { 
                coords, 
                image: Image { 
                    inner: tile.into_rgba8() 
                }
            })
            .collect()
    }
    
    pub fn from_tiles(w: u32, h: u32, tile_width: u32, tile_height: u32, tiles: Vec<Tile<Image>>) -> Image {
        let mut dyn_img = DynamicImage::new_rgba8(w, h);

        for tile in tiles {
            let dyn_tile: DynamicImage = tile.image.inner.into();
            imageops::overlay(&mut dyn_img, &dyn_tile, (tile.coords.0*tile_width) as i64, (tile.coords.1*tile_height) as i64);
        }

        Image { inner: dyn_img.into_rgba8() }
    }

    pub fn width(&self) -> u32 {
        self.inner.width()
    }
    
    pub fn height(&self) -> u32 {
        self.inner.height()
    }
    
}