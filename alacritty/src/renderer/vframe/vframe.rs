use std::fs::read_to_string;
use std::path::PathBuf;
use std::time::{Instant};

use crate::gl::types::GLint;
use crate::config::UiConfig;
use crate::renderer::vframe::ShaderRawPro;
use crate::{display::SizeInfo, gl::{self, types::{ GLuint }}};
use crate::config;
use std::time::Duration;
use crate::cli::Options;
#[derive(Debug, Clone)]
pub struct MyFramebuffer {
    pub fbo_id: gl::types::GLuint,     // The FBO itself
    pub texture_id: gl::types::GLuint, // The texture we draw *into*
    pub rbo_id: gl::types::GLuint,     // The depth/stencil buffer
    pub fullscreen_vao: GLuint,
    pub width: i32,
    pub height: i32,
    program: GLuint,
    start_time: Instant,
    loc : GLint,
    current_loc: GLint,
    resolution_loc: GLint,
    previous_loc: GLint ,
    color_loc: GLint ,
    prev_color_loc: GLint,
    time_loc: GLint,
    change_loc: GLint,
    duration_loc: GLint,
    pub is_cprogram_loaded: bool,
    last_frame_time: f32,
    last_cursor_change: f32,
    pub stop_animated: bool,
    from_x: f32,
    from_y: f32,
    target_x: f32,
    target_y: f32,
    render_x: f32,
    render_y: f32,
    vel_x: f32,
    vel_y: f32,
    pub config: UiConfig,

}

impl Drop for MyFramebuffer {
    fn drop(&mut self) {
        // This code automatically runs when the struct is dropped,
        // (like when you replace it during a window resize)
        unsafe {
            gl::DeleteFramebuffers(1, &self.fbo_id);
            gl::DeleteTextures(1, &self.texture_id);
            gl::DeleteRenderbuffers(1, &self.rbo_id);
            gl::DeleteVertexArrays(1, &self.fullscreen_vao);
        }
    }
}
// ➡️ This array defines two triangles that form a quad covering the entire [-1.0, 1.0] NDC space.
// const FULL_SCREEN_VERTICES: [f32; 24] = [
//     // Position (X, Y),   TexCoords (U, V)
//     -1.0,  1.0,  0.0, 1.0, // Top-Left (V1)
//     -1.0, -1.0,  0.0, 0.0, // Bottom-Left (V2)
//      1.0, -1.0,  1.0, 0.0, // Bottom-Right (V3)
//
//     -1.0,  1.0,  0.0, 1.0, // Top-Left (V1) - Reused
//      1.0, -1.0,  1.0, 0.0, // Bottom-Right (V3) - Reused
//      1.0,  1.0,  1.0, 1.0, // Top-Right (V4)
// ];
// fn update_projection(u_projection: GLint, size: &SizeInfo) {
//     let width = size.width();
//     let height = size.height();
//     let padding_x = size.padding_x();
//     let padding_y = size.padding_y();
//
//     // Bounds check.
//     if (width as u32) < (2 * padding_x as u32) || (height as u32) < (2 * padding_y as u32) {
//         return;
//     }
//
//     // Compute scale and offset factors, from pixel to ndc space. Y is inverted.
//     //   [0, width - 2 * padding_x] to [-1, 1]
//     //   [height - 2 * padding_y, 0] to [-1, 1]
//     let scale_x = 2. / (width - 2. * padding_x);
//     let scale_y = -2. / (height - 2. * padding_y);
//     let offset_x = -1.;
//     let offset_y = 1.;
//
//     unsafe {
//         gl::Uniform4f(u_projection, offset_x, offset_y, scale_x, scale_y);
//     }
// }
//
const DEFAULT_VERTEX: &str = include_str!("c.v.glsl");
const DEFAULT_FRAGMENT: &str = include_str!("c.f.glsl");
/// Creates a new Framebuffer Object (FBO) with a texture to draw into.
///
/// `width` and `height` should be the size of your window.
impl MyFramebuffer {
    pub fn create_framebuffer(
        width: i32,
        height: i32,
        // vao: GLuint,
        // vbo: GLuint,
    ) -> Result<Self, String> {
        let mut program_use_id: GLuint = 0;
        let mut is_cprogram_loaded: bool = false;
        let mut options = Options::new();
        let config = config::load(&mut options);

        println!("shader: {}", config.general.shader.clone());
        let program_default = ShaderRawPro::new(DEFAULT_VERTEX.to_string(), DEFAULT_FRAGMENT.to_string());
        let mut fragment_path: PathBuf = PathBuf::from(config.general.shader.clone());
        if fragment_path.is_relative() {
            for file in &config.config_paths {
                if let Some(config_path) = file.parent() {
                    let fpath = config_path;
                    let full_fpath = fpath.join(&fragment_path);
                    if full_fpath.exists() {
                        fragment_path = full_fpath;
                        break;
                    }
                }
            }
        }
        match read_to_string(&fragment_path) {
            Ok(fsrc) =>  {
                let program = ShaderRawPro::new(DEFAULT_VERTEX.to_string(), fsrc);
                match program {
                    Ok(pro) => {

                        program_use_id = pro.id();
                        is_cprogram_loaded = true;
                    },
                    Err(ref e) => {
                        if let Ok(pd) =  program_default {
                            program_use_id = pd.id();
                        }
                        println!("ShaderErr: {}", e);
                    }
                }
            },
            Err(e) => {
                println!("read_to_string {}: {}",fragment_path.display(), e);
                if let Ok(pd) =  program_default {
                    program_use_id = pd.id();
                }
            }
        }
        let loc = unsafe { gl::GetUniformLocation(program_use_id, "iChannel0\0".as_ptr() as *const i8) };
        let current_loc = unsafe { gl::GetUniformLocation(program_use_id, "iCurrentCursor\0".as_ptr() as *const i8) };
        let resolution_loc = unsafe { gl::GetUniformLocation(program_use_id, "iResolution\0".as_ptr() as *const i8) };
        let previous_loc = unsafe { gl::GetUniformLocation(program_use_id, "iPreviousCursor\0".as_ptr() as *const i8) };
        let color_loc = unsafe { gl::GetUniformLocation(program_use_id, "iCurrentCursorColor\0".as_ptr() as *const i8) };
        let prev_color_loc = unsafe { gl::GetUniformLocation(program_use_id, "iPreviousCursorColor\0".as_ptr() as *const i8) };
        let time_loc = unsafe { gl::GetUniformLocation(program_use_id, "iTime\0".as_ptr() as *const i8) };
        let change_loc = unsafe { gl::GetUniformLocation(program_use_id, "iTimeCursorChange\0".as_ptr() as *const i8) };
        let duration_loc = unsafe { gl::GetUniformLocation(program_use_id, b"iDuration\0".as_ptr() as *const i8) };
        let (fbo_id, texture_id, rbo_id, fullscreen_vao) = Self::setup_vframe(width, height);
        Ok(MyFramebuffer {
            fbo_id: fbo_id,
            texture_id: texture_id,
            rbo_id: rbo_id ,
            width: width,
            height: height,
            last_cursor_change: 0.0,
            program: program_use_id,
            start_time: Instant::now(),
            loc: loc,
            current_loc: current_loc,
            resolution_loc:resolution_loc,
            previous_loc: previous_loc,
            color_loc: color_loc,
            prev_color_loc:
            prev_color_loc,
            time_loc: time_loc,
            change_loc: change_loc,
            duration_loc: duration_loc,
            is_cprogram_loaded: is_cprogram_loaded,
            from_x: 0.0,
            from_y: 0.0,
            render_x: 0.0,
            render_y: 0.0,
            target_x: 0.0,
            target_y: 0.0,
            last_frame_time: 0.0,
            vel_x: 0.0,
            vel_y: 0.0,
            config: config,
            stop_animated: true,
            fullscreen_vao: fullscreen_vao,
        })
    }

    pub fn setup_vframe(width: i32, height: i32) -> (GLuint, GLuint, GLuint, GLuint) {
        let mut fullscreen_vao: GLuint = 0;

        unsafe {
          gl::GenVertexArrays(1, &mut fullscreen_vao);
        }
        let (mut fbo_id, mut texture_id, mut rbo_id) = (0, 0, 0);


        unsafe {
            // 1. --- CREATE THE FRAMEBUFFER ---
            gl::GenFramebuffers(1, &mut fbo_id);
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo_id);

            // 2. --- CREATE THE COLOR TEXTURE ---
            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);

            // Create an *empty* texture. We give it `null` data.
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                width,
                height,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                std::ptr::null() as *const gl::types::GLvoid, // No data
            );

            // Set filters (required!)
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            // Attach the texture to the FBO
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0, // Attach it as color target
                gl::TEXTURE_2D,
                texture_id,
                0, // Mipmap level
            );

            // 3. --- CREATE THE DEPTH/STENCIL RENDERBUFFER ---
            gl::GenRenderbuffers(1, &mut rbo_id);
            gl::BindRenderbuffer(gl::RENDERBUFFER, rbo_id);

            // Allocate storage for depth and stencil
            gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, width, height);

            // Attach the renderbuffer to the FBO
            gl::FramebufferRenderbuffer(
                gl::FRAMEBUFFER,
                gl::DEPTH_STENCIL_ATTACHMENT, // Attach as depth/stencil
                gl::RENDERBUFFER,
                rbo_id,
            );

            // 4. --- CHECK IF IT'S COMPLETE ---
            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                return (0, 0, 0, 0);
            }

            // 5. --- UNBIND ---
            // Unbind the FBO so we don't accidentally draw to it
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);


        }
        (fbo_id, texture_id, rbo_id, fullscreen_vao)
    }
    fn update_cursor(&mut self, dt: f32) {
        let stiffness = 420.0;
        let damping = 26.0;

        let ax = (self.target_x - self.render_x) * stiffness;
        let ay = (self.target_y - self.render_y) * stiffness;

        self.vel_x = (self.vel_x + ax * dt) * (-damping * dt).exp();
        self.vel_y = (self.vel_y + ay * dt) * (-damping * dt).exp();

        self.render_x += self.vel_x * dt;
        self.render_y += self.vel_y * dt;

        if (self.target_x - self.render_x).abs() < 0.01 && self.vel_x.abs() < 0.01 {
            self.render_x = self.target_x;
            self.vel_x = 0.0;
        }

        if (self.target_y - self.render_y).abs() < 0.01 && self.vel_y.abs() < 0.01 {
            self.render_y = self.target_y;
            self.vel_y = 0.0;
        }
    }

    pub fn update_render_data(
        &mut self,
        size_info: &SizeInfo,
        cursor_pos_x: f32,
        cursor_pos_y: f32,
    ) {
        let now = self.start_time.elapsed().as_secs_f32();
        let cellw = size_info.cell_width();
        let cellh = size_info.cell_height();

        // first frame
        if self.last_frame_time == 0.0 {
            self.from_x = cursor_pos_x;
            self.from_y = cursor_pos_y;
            self.target_x = cursor_pos_x;
            self.target_y = cursor_pos_y;
            self.render_x = cursor_pos_x;
            self.render_y = cursor_pos_y;
            self.last_frame_time = now;
            self.last_cursor_change = now;
            self.stop_animated = false;
        }

        // cursor moved -> restart animation
        if cursor_pos_x != self.target_x || cursor_pos_y != self.target_y {
            self.from_x = self.render_x;
            self.from_y = self.render_y;
            self.target_x = cursor_pos_x;
            self.target_y = cursor_pos_y;
            self.last_cursor_change = now;
            self.stop_animated = false;
        }

        let dt = (now - self.last_frame_time).clamp(0.0, 1.0 / 30.0);
        self.last_frame_time = now;

        if !self.stop_animated {
            self.update_cursor(dt);
        }

        if !self.stop_animated && (now - self.last_cursor_change) >= self.config.general.shader_duration
        {
            self.render_x = self.target_x;
            self.render_y = self.target_y;
            self.vel_x = 0.0;
            self.vel_y = 0.0;
            self.stop_animated = true;
            println!("timeout reached, stop animating !");
        }

        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Disable(gl::BLEND);
            gl::Viewport(0, 0, size_info.width() as i32, size_info.height() as i32);
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::BindVertexArray(self.fullscreen_vao);
            gl::UseProgram(self.program);

            gl::Uniform2f(self.resolution_loc, size_info.width() as f32, size_info.height() as f32);
            gl::Uniform4f(self.previous_loc, self.from_x, self.from_y, cellw, cellh);
            gl::Uniform4f(self.current_loc, self.render_x, self.render_y, cellw, cellh);

            gl::Uniform4f(self.color_loc, 0.65, 0.6, 0.7, 1.0);
            gl::Uniform4f(self.prev_color_loc, 0.65, 0.6, 0.7, 1.0);
            gl::Uniform1f(self.time_loc, now);
            gl::Uniform1f(self.change_loc, self.last_cursor_change);
            gl::Uniform1f(self.duration_loc, self.config.general.shader_duration);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture_id);
            gl::Uniform1i(self.loc, 0);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }
    }

    pub fn resize_vframe(
        &mut self,
    new_width: i32,
    new_height: i32,
) {
    unsafe {
        // 1. --- RESIZE THE COLOR TEXTURE ---
        gl::BindTexture(gl::TEXTURE_2D, self.texture_id);

        // ➡️ Re-allocate its storage with the new size (pass NULL for data)
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            new_width,
            new_height,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            std::ptr::null() as *const gl::types::GLvoid, // No data
        );
        gl::BindTexture(gl::TEXTURE_2D, 0); // Unbind

        // 2. --- RESIZE THE DEPTH/STENCIL RENDERBUFFER ---
        gl::BindRenderbuffer(gl::RENDERBUFFER, self.rbo_id);

        // ➡️ Re-allocate its storage with the new size
        gl::RenderbufferStorage(
            gl::RENDERBUFFER,
            gl::DEPTH24_STENCIL8,
            new_width,
            new_height,
        );
        gl::BindRenderbuffer(gl::RENDERBUFFER, 0); // Unbind
        self.width = new_width;
        self.height = new_height;
    }
}
    pub fn start(&self) {
        unsafe {
            // ➡️ THIS IS THE COMMAND YOU ARE MISSING
            // Tell OpenGL to stop drawing to the screen and
            // draw to *your* fbo_id instead.
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo_id);
            gl::Viewport(0, 0, self.width as i32, self.height as i32);

        }
    }
    pub fn needs_redraw(&self) -> bool {
        !self.stop_animated || self.config.general.animated
    }
    pub fn redraw_interval(&self) -> Duration {
        let fps = self.config.general.vframe_fps.max(1);
        Duration::from_secs_f64(1.0 / fps as f64)
    }
    pub fn stop(&self) {
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0) };
    }
    // pub fn clear(&self) {
    //     let color = self.config.colors.primary.background;
    //     let alpha = self.config.window_opacity();
    //     unsafe {
    //         gl::ClearColor(
    //             (f32::from(color.r) / 255.0).min(1.0) * alpha,
    //             (f32::from(color.g) / 255.0).min(1.0) * alpha,
    //             (f32::from(color.b) / 255.0).min(1.0) * alpha,
    //             alpha,
    //         );
    //         gl::Clear(gl::COLOR_BUFFER_BIT);
    //     }
    // }
}
