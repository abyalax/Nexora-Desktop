use crate::gfx::Gfx;
use windows::Graphics::Capture::{Direct3D11CaptureFramePool, GraphicsCaptureItem, GraphicsCaptureSession};
use windows::Graphics::DirectX::Direct3D11::IDirect3DDevice;
use windows::Graphics::DirectX::DirectXPixelFormat;
use windows::Graphics::SizeInt32;
use windows::UI::Composition::CompositionDrawingSurface;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Direct3D11::ID3D11Texture2D;
use windows::Win32::System::WinRT::Direct3D11::IDirect3DDxgiInterfaceAccess;
use windows::Win32::System::WinRT::Graphics::Capture::IGraphicsCaptureItemInterop;
use windows_core::{Interface, Result};

/// Captures a live desktop rectangle and encodes it as a PNG.
pub fn screenshot_png(rect: (i32, i32, i32, i32)) -> std::io::Result<Vec<u8>> {
  use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC,
    DeleteObject, GetDC, GetDIBits, ReleaseDC, SRCCOPY, SelectObject,
  };
  let (x, y, w, h) = rect;
  if w <= 0 || h <= 0 || w > 16384 || h > 16384 {
    return Err(std::io::Error::new(
      std::io::ErrorKind::InvalidInput,
      "invalid capture rectangle",
    ));
  }
  unsafe {
    let screen = GetDC(None);
    if screen.is_invalid() {
      return Err(std::io::Error::last_os_error());
    }
    let memory = CreateCompatibleDC(Some(screen));
    let bitmap = CreateCompatibleBitmap(screen, w, h);
    if memory.is_invalid() || bitmap.is_invalid() {
      if !memory.is_invalid() {
        let _ = DeleteDC(memory);
      }
      let _ = ReleaseDC(None, screen);
      return Err(std::io::Error::last_os_error());
    }
    let old = SelectObject(memory, bitmap.into());
    let copied = BitBlt(memory, 0, 0, w, h, Some(screen), x, y, SRCCOPY).is_ok();
    let _ = SelectObject(memory, old);
    let mut pixels = vec![0u8; (w as usize) * (h as usize) * 4];
    let mut info = BITMAPINFO {
      bmiHeader: BITMAPINFOHEADER {
        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: w,
        biHeight: -h,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB.0,
        ..Default::default()
      },
      ..Default::default()
    };
    let rows = if copied {
      GetDIBits(
        memory,
        bitmap,
        0,
        h as u32,
        Some(pixels.as_mut_ptr().cast()),
        &mut info,
        DIB_RGB_COLORS,
      )
    } else {
      0
    };
    let _ = DeleteObject(bitmap.into());
    let _ = DeleteDC(memory);
    let _ = ReleaseDC(None, screen);
    if rows != h {
      return Err(std::io::Error::last_os_error());
    }
    let mut rgba = pixels;
    for p in rgba.chunks_exact_mut(4) {
      p.swap(0, 2);
      p[3] = 255;
    }
    let mut output = Vec::new();
    let mut encoder = png::Encoder::new(&mut output, w as u32, h as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(std::io::Error::other)?;
    writer.write_image_data(&rgba).map_err(std::io::Error::other)?;
    drop(writer);
    Ok(output)
  }
}

pub fn supported() -> bool {
  GraphicsCaptureSession::IsSupported().unwrap_or(false)
}

pub struct Capture {
  pool: Direct3D11CaptureFramePool,
  session: GraphicsCaptureSession,
  device: IDirect3DDevice,
  size: SizeInt32,
  pub surface: CompositionDrawingSurface,
  pub has_frame: bool,
}

impl Capture {
  pub fn start(gfx: &Gfx, hwnd: isize) -> Result<Capture> {
    let interop = windows_core::factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()?;
    let item: GraphicsCaptureItem = unsafe { interop.CreateForWindow(HWND(hwnd as *mut _))? };
    let size = item.Size()?;
    if size.Width <= 0 || size.Height <= 0 {
      return Err(windows_core::Error::empty());
    }
    let device = gfx.d3d_device()?;
    let pool =
      Direct3D11CaptureFramePool::CreateFreeThreaded(&device, DirectXPixelFormat::B8G8R8A8UIntNormalized, 2, size)?;
    let session = pool.CreateCaptureSession(&item)?;
    let _ = session.SetIsCursorCaptureEnabled(false);
    let _ = session.SetIsBorderRequired(false);
    session.StartCapture()?;
    let surface = gfx.new_surface(size.Width as u32, size.Height as u32)?;
    Ok(Capture {
      pool,
      session,
      device,
      size,
      surface,
      has_frame: false,
    })
  }

  pub fn size(&self) -> (i32, i32) {
    (self.size.Width, self.size.Height)
  }

  pub fn poll(&mut self, gfx: &Gfx) -> bool {
    let mut last = None;
    while let Ok(f) = self.pool.TryGetNextFrame() {
      last = Some(f);
    }
    let Some(frame) = last else { return false };
    let Ok(cs) = frame.ContentSize() else {
      return false;
    };
    if cs.Width <= 0 || cs.Height <= 0 {
      return false;
    }
    if cs.Width != self.size.Width || cs.Height != self.size.Height {
      let _ = self
        .pool
        .Recreate(&self.device, DirectXPixelFormat::B8G8R8A8UIntNormalized, 2, cs);
      self.size = cs;
    }
    let ok = (|| -> Result<()> {
      let surf = frame.Surface()?;
      let access: IDirect3DDxgiInterfaceAccess = surf.cast()?;
      let tex: ID3D11Texture2D = unsafe { access.GetInterface()? };
      gfx.copy_into(&self.surface, &tex, cs.Width as u32, cs.Height as u32)
    })()
    .is_ok();
    let _ = frame.Close();
    if ok {
      self.has_frame = true;
    }
    ok
  }
}

impl Drop for Capture {
  fn drop(&mut self) {
    let _ = self.session.Close();
    let _ = self.pool.Close();
  }
}
