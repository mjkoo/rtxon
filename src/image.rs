use core::marker::PhantomData;
use std::alloc::{alloc, dealloc, Layout};
use std::io::Result;
use std::mem::size_of;
use std::path::Path;

use image::{ImageBuffer, Pixel};
use raw_cpuid::CpuId;
use scoped_threadpool::Pool;

fn cache_line_size() -> Option<usize> {
    let cpuid = CpuId::new();
    if let Some(cparams) = cpuid.get_cache_parameters() {
        if let Some(cparam) = cparams.filter(|c| c.level() == 1).take(1).next() {
            return Some(cparam.coherency_line_size());
        }
    }

    None
}

// Each row should only be accessed by one thread
struct Row<P: Pixel>(*mut P);
unsafe impl<P: Pixel> Send for Row<P> {}
unsafe impl<P: Pixel> Sync for Row<P> {}

pub struct Image<P: Pixel> {
    width: usize,
    height: usize,
    row_layout: Layout,
    rows: Vec<Row<P>>,
    _marker: PhantomData<P>,
}

impl<P: Pixel> Image<P> {
    pub fn new(width: usize, height: usize) -> Self {
        let bytes_per_pixel = size_of::<P::Subpixel>() * (P::channel_count() as usize);
        let bytes_per_row = bytes_per_pixel * width;
        let align = cache_line_size();

        let row_layout = Layout::from_size_align(bytes_per_row, align.unwrap_or(64))
            .expect("invalid memory layout");

        // Allocate each row of the image as aligned memory to prevent false sharing
        let mut rows = vec![];
        unsafe {
            for _ in 0..height {
                rows.push(Row(alloc(row_layout) as *mut P));
            }
        }

        Self {
            width,
            height,
            row_layout,
            rows,
            _marker: Default::default(),
        }
    }

    pub fn render<F>(&mut self, f: F)
        where F: Fn(u32, u32) -> P + Send + Clone + 'static
    {
        let nproc = num_cpus::get();
        let mut pool = Pool::new(nproc as u32);

        // Schedule rendering of each row on our threadpool
        pool.scoped(|scoped| {
            let width = self.width;

            for (y, row) in self.rows.iter().enumerate() {
                let f = f.clone();
                scoped.execute(move || {
                    unsafe {
                        for x in 0..width {
                            *row.0.offset(x as isize) = f(x as u32, y as u32);
                        }
                    }
                })
            }
        });
    }

    pub fn from_fn<F>(width: usize, height: usize, f: F) -> Self
        where F: Fn(u32, u32) -> P + Send + Clone + 'static 
    {
        let mut img = Self::new(width, height);
        img.render(f);
        img
    }
}

impl<P: Pixel<Subpixel=u8> + 'static> Image<P> {
    pub fn save<Q>(&self, path: Q) -> Result<()>
    where
        Q: AsRef<Path>,
    {
        // Construct an ImageBuffer from each row of pixels
        let img = ImageBuffer::from_fn(self.width as u32, self.height as u32, |x, y| -> P {
            let row = &self.rows[y as usize];
            unsafe {
                let (a, b, c, d) = (*row.0.offset(x as isize)).channels4();
                P::from_channels(a, b, c, d)
            }
        });

        img.save(path)
    }
}

impl<P: Pixel> Drop for Image<P> {
    fn drop(&mut self) {
        unsafe {
            for r in self.rows.iter() {
                dealloc(r.0 as *mut u8, self.row_layout);
            }
        }
    }
}
