use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::*;
use nalgebra_glm as glm;

// console.logのラッパー
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// 3Dビューアの状態を管理する構造体
#[wasm_bindgen]
pub struct GltfViewer {
    gl: WebGl2RenderingContext,
    program: WebGlProgram,
    vertex_buffer: WebGlBuffer,
    index_buffer: WebGlBuffer,
    index_count: i32,
    // カメラ関連
    view_matrix: glm::Mat4,
    projection_matrix: glm::Mat4,
    camera_position: glm::Vec3,
    camera_target: glm::Vec3,
    // uniform locations
    u_mvp_matrix: WebGlUniformLocation,
    u_color: WebGlUniformLocation,
}

#[wasm_bindgen]
impl GltfViewer {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<GltfViewer, JsValue> {
        console_error_panic_hook::set_once();
        console_log!("Initializing GLTF Viewer...");
        
        // Canvasを取得
        let window = window().unwrap();
        let document = window.document().unwrap();
        let canvas = document
            .get_element_by_id(canvas_id)
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()?;
        
        // WebGL2コンテキストを取得
        let gl = canvas
            .get_context("webgl2")?
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()?;
        
        // シェーダープログラムを作成
        let vertex_shader_source = r#"#version 300 es
            in vec3 a_position;
            uniform mat4 u_mvp_matrix;
            
            void main() {
                gl_Position = u_mvp_matrix * vec4(a_position, 1.0);
            }
        "#;
        
        let fragment_shader_source = r#"#version 300 es
            precision mediump float;
            uniform vec3 u_color;
            out vec4 fragColor;
            
            void main() {
                fragColor = vec4(u_color, 1.0);
            }
        "#;
        
        let program = Self::create_program(&gl, vertex_shader_source, fragment_shader_source)?;
        
        // uniform locationを取得
        let u_mvp_matrix = gl.get_uniform_location(&program, "u_mvp_matrix")
            .ok_or("Failed to get u_mvp_matrix uniform location")?;
        let u_color = gl.get_uniform_location(&program, "u_color")
            .ok_or("Failed to get u_color uniform location")?;
        
        // バッファを作成
        let vertex_buffer = gl.create_buffer()
            .ok_or("Failed to create vertex buffer")?;
        let index_buffer = gl.create_buffer()
            .ok_or("Failed to create index buffer")?;
        
        // カメラ設定
        let camera_position = glm::vec3(3.0, 3.0, 5.0);
        let camera_target = glm::vec3(0.0, 0.0, 0.0);
        let up = glm::vec3(0.0, 1.0, 0.0);
        
        let view_matrix = glm::look_at(&camera_position, &camera_target, &up);
        let projection_matrix = glm::perspective(
            canvas.width() as f32 / canvas.height() as f32,
            45.0_f32.to_radians(),
            0.1,
            100.0,
        );
        
        // WebGL設定
        gl.enable(WebGl2RenderingContext::DEPTH_TEST);
        gl.clear_color(0.1, 0.1, 0.1, 1.0);
        
        console_log!("GLTF Viewer initialized successfully");
        
        Ok(GltfViewer {
            gl,
            program,
            vertex_buffer,
            index_buffer,
            index_count: 0,
            view_matrix,
            projection_matrix,
            camera_position,
            camera_target,
            u_mvp_matrix,
            u_color,
        })
    }
    
    // テスト用の立方体を作成
    #[wasm_bindgen]
    pub fn create_test_box(&mut self) -> Result<(), JsValue> {
        console_log!("Creating test box...");
        
        // 立方体の頂点データ
        let vertices: [f32; 24] = [
            // 前面
            -1.0, -1.0,  1.0,
             1.0, -1.0,  1.0,
             1.0,  1.0,  1.0,
            -1.0,  1.0,  1.0,
            // 後面
            -1.0, -1.0, -1.0,
            -1.0,  1.0, -1.0,
             1.0,  1.0, -1.0,
             1.0, -1.0, -1.0,
        ];
        
        // インデックスデータ
        let indices: [u16; 36] = [
            0, 1, 2, 0, 2, 3,    // 前面
            4, 5, 6, 4, 6, 7,    // 後面
            4, 0, 3, 4, 3, 5,    // 左面
            1, 7, 6, 1, 6, 2,    // 右面
            3, 2, 6, 3, 6, 5,    // 上面
            4, 7, 1, 4, 1, 0,    // 下面
        ];
        
        // 頂点バッファにデータをアップロード
        self.gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.vertex_buffer));
        
        unsafe {
            let vertices_array = js_sys::Float32Array::view(&vertices);
            self.gl.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &vertices_array,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }
        
        // インデックスバッファにデータをアップロード
        self.gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&self.index_buffer));
        
        unsafe {
            let indices_array = js_sys::Uint16Array::view(&indices);
            self.gl.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
                &indices_array,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }
        self.index_count = 36;
        
        console_log!("Test box created");
        Ok(())
    }
    
    // シーンをレンダリング
    #[wasm_bindgen]
    pub fn render(&mut self) -> Result<(), JsValue> {
        if self.index_count == 0 {
            return Ok(()); // ジオメトリがない場合は何もしない
        }
        
        // 画面をクリア
        self.gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT);
        
        // シェーダープログラムを使用
        self.gl.use_program(Some(&self.program));
        
        // MVP行列を計算
        let model_matrix = glm::Mat4::identity();
        let mvp_matrix = self.projection_matrix * self.view_matrix * model_matrix;
        
        // ユニフォームを設定
        self.gl.uniform_matrix4fv_with_f32_array(
            Some(&self.u_mvp_matrix),
            false,
            mvp_matrix.as_slice(),
        );
        
        self.gl.uniform3f(Some(&self.u_color), 0.8, 0.4, 0.2); // オレンジ色
        
        // 頂点属性を設定
        self.gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.vertex_buffer));
        self.gl.vertex_attrib_pointer_with_i32(0, 3, WebGl2RenderingContext::FLOAT, false, 0, 0);
        self.gl.enable_vertex_attrib_array(0);
        
        // インデックスバッファをバインド
        self.gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&self.index_buffer));
        
        // 描画
        self.gl.draw_elements_with_i32(
            WebGl2RenderingContext::TRIANGLES,
            self.index_count,
            WebGl2RenderingContext::UNSIGNED_SHORT,
            0,
        );
        
        Ok(())
    }
    
    // カメラを回転
    #[wasm_bindgen]
    pub fn rotate_camera(&mut self, delta_x: f32, delta_y: f32) {
        let distance = glm::length(&(self.camera_position - self.camera_target));
        
        // 球面座標でカメラを回転
        let to_target = self.camera_position - self.camera_target;
        let phi = to_target.z.atan2(to_target.x) + delta_x * 0.01;
        let theta = (to_target.y / distance).acos() + delta_y * 0.01;
        
        let theta = theta.max(0.1).min(std::f32::consts::PI - 0.1);
        
        self.camera_position = self.camera_target + glm::vec3(
            distance * theta.sin() * phi.cos(),
            distance * theta.cos(),
            distance * theta.sin() * phi.sin(),
        );
        
        let up = glm::vec3(0.0, 1.0, 0.0);
        self.view_matrix = glm::look_at(&self.camera_position, &self.camera_target, &up);
    }
    
    // ビューポートサイズを更新
    #[wasm_bindgen]
    pub fn resize(&mut self, width: u32, height: u32) {
        self.gl.viewport(0, 0, width as i32, height as i32);
        self.projection_matrix = glm::perspective(
            width as f32 / height as f32,
            45.0_f32.to_radians(),
            0.1,
            100.0,
        );
    }
    
    // GLTFファイルを読み込む
    #[wasm_bindgen]
    pub fn load_gltf(&mut self, gltf_data: &[u8]) -> Result<(), JsValue> {
        console_log!("Loading GLTF data... {} bytes", gltf_data.len());
        
        // まず基本的なGLTFファイルの検証
        if gltf_data.len() < 4 {
            return Err(JsValue::from_str("GLTF file too small"));
        }
        
        // GLBファイルかどうかチェック（最初の4バイトが"glTF"）
        let is_glb = &gltf_data[0..4] == b"glTF";
        console_log!("File type: {}", if is_glb { "GLB (binary)" } else { "GLTF (JSON)" });
        
        // GLTFファイルをパース
        let result = if is_glb {
            // GLBファイルの場合
            gltf::import_slice(gltf_data)
        } else {
            // JSONファイルの場合、文字列として解析を試行
            match std::str::from_utf8(gltf_data) {
                Ok(json_str) => {
                    console_log!("Parsing as JSON GLTF, {} characters", json_str.len());
                    gltf::import_slice(gltf_data)
                }
                Err(e) => {
                    console_log!("Not valid UTF-8, treating as binary: {:?}", e);
                    gltf::import_slice(gltf_data)
                }
            }
        };
        
        let (gltf, buffers, _images) = result.map_err(|e| {
            console_log!("GLTF import error details: {:?}", e);
            let error_msg = format!("Failed to import GLTF file: {}", e);
            console_log!("Error message: {}", error_msg);
            JsValue::from_str(&error_msg)
        })?;
        
        console_log!("GLTF imported successfully!");
        console_log!("- Scenes: {}", gltf.scenes().count());
        console_log!("- Meshes: {}", gltf.meshes().count());
        console_log!("- Buffers: {}", buffers.len());
        console_log!("- Nodes: {}", gltf.nodes().count());
        
        if gltf.meshes().count() == 0 {
            console_log!("No meshes found in GLTF file, creating fallback box");
            return self.create_test_box();
        }
        
        // 既存のジオメトリをクリア
        self.clear_geometry();
        
        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();
        let mut index_offset = 0u16;
        
        // 各メッシュを処理
        for (mesh_index, mesh) in gltf.meshes().enumerate() {
            console_log!("Processing mesh {}: {}", mesh_index, mesh.name().unwrap_or("unnamed"));
            
            for (prim_index, primitive) in mesh.primitives().enumerate() {
                console_log!("  Processing primitive {}", prim_index);
                match self.process_primitive(&primitive, &buffers) {
                    Ok(Some((vertices, indices))) => {
                        // インデックスをオフセット調整して追加
                        let adjusted_indices: Vec<u16> = indices.iter()
                            .map(|&i| i + index_offset)
                            .collect();
                        
                        all_vertices.extend_from_slice(&vertices);
                        all_indices.extend_from_slice(&adjusted_indices);
                        index_offset += (vertices.len() / 3) as u16;
                        
                        console_log!("    Added {} vertices, {} indices", vertices.len() / 3, indices.len());
                    }
                    Ok(None) => {
                        console_log!("    Primitive {} skipped (no geometry)", prim_index);
                    }
                    Err(e) => {
                        console_log!("    Error processing primitive {}: {:?}", prim_index, e);
                        // エラーがあっても他のプリミティブを処理し続ける
                    }
                }
            }
        }
        
        if all_vertices.is_empty() {
            console_log!("No geometry extracted from GLTF, creating fallback box");
            return self.create_test_box();
        }
        
        console_log!("Total vertices: {}, Total indices: {}", all_vertices.len() / 3, all_indices.len());
        
        // バッファにデータをアップロード
        self.upload_geometry(&all_vertices, &all_indices)?;
        
        console_log!("GLTF loading completed successfully");
        Ok(())
    }
    
    // プリミティブを処理してジオメトリを取得
    fn process_primitive(
        &mut self, 
        primitive: &gltf::Primitive, 
        buffers: &[gltf::buffer::Data]
    ) -> Result<Option<(Vec<f32>, Vec<u16>)>, JsValue> {
        console_log!("    Processing primitive with mode: {:?}", primitive.mode());
        
        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
        
        // 位置データを取得
        let positions = match reader.read_positions() {
            Some(pos_iter) => pos_iter.collect::<Vec<[f32; 3]>>(),
            None => {
                console_log!("    No position data found in primitive");
                return Ok(None);
            }
        };
        
        console_log!("    Found {} positions in primitive", positions.len());
        
        // 三角形以外のプリミティブタイプをチェック
        if primitive.mode() != gltf::mesh::Mode::Triangles {
            console_log!("    Warning: Non-triangle primitive mode: {:?}", primitive.mode());
            // 三角形以外でも処理を続行
        }
        
        // 頂点データを平坦化
        let vertices: Vec<f32> = positions.iter()
            .flat_map(|pos| pos.iter().cloned())
            .collect();
        
        // インデックスデータを取得
        let indices: Vec<u16> = if let Some(indices_reader) = reader.read_indices() {
            match indices_reader {
                gltf::mesh::util::ReadIndices::U8(iter) => {
                    console_log!("    Using U8 indices");
                    iter.map(|i| i as u16).collect()
                },
                gltf::mesh::util::ReadIndices::U16(iter) => {
                    console_log!("    Using U16 indices");
                    iter.collect()
                },
                gltf::mesh::util::ReadIndices::U32(iter) => {
                    console_log!("    Using U32 indices (converting to U16)");
                    iter.map(|i| {
                        if i > u16::MAX as u32 {
                            console_log!("    Warning: Index {} exceeds u16::MAX, clamping", i);
                            u16::MAX
                        } else {
                            i as u16
                        }
                    }).collect()
                },
            }
        } else {
            // インデックスがない場合は順番に生成
            console_log!("    No indices found, generating sequential indices");
            (0..positions.len() as u16).collect()
        };
        
        console_log!("    Generated {} indices for primitive", indices.len());
        
        // 基本的な検証
        if vertices.is_empty() {
            console_log!("    Warning: Empty vertices array");
            return Ok(None);
        }
        
        if indices.is_empty() {
            console_log!("    Warning: Empty indices array");
            return Ok(None);
        }
        
        Ok(Some((vertices, indices)))
    }
    
    // ジオメトリをクリア
    fn clear_geometry(&mut self) {
        // 現在のジオメトリをクリアするために空のバッファを作成
        self.gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.vertex_buffer));
        self.gl.buffer_data_with_i32(
            WebGl2RenderingContext::ARRAY_BUFFER,
            0,
            WebGl2RenderingContext::STATIC_DRAW,
        );
        
        self.gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&self.index_buffer));
        self.gl.buffer_data_with_i32(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            0,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }
    
    // ジオメトリデータをGPUにアップロード
    fn upload_geometry(&mut self, vertices: &[f32], indices: &[u16]) -> Result<(), JsValue> {
        // 頂点バッファにデータをアップロード
        self.gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.vertex_buffer));
        
        unsafe {
            let vertices_array = js_sys::Float32Array::view(vertices);
            self.gl.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &vertices_array,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }
        
        // インデックスバッファにデータをアップロード
        self.gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&self.index_buffer));
        
        unsafe {
            let indices_array = js_sys::Uint16Array::view(indices);
            self.gl.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
                &indices_array,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }
        
        // レンダリング時に使用するインデックス数を保存
        self.index_count = indices.len() as i32;
        
        console_log!("Uploaded geometry: {} vertices, {} indices", vertices.len() / 3, indices.len());
        
        Ok(())
    }
    
    // シェーダープログラムを作成
    fn create_program(
        gl: &WebGl2RenderingContext,
        vertex_source: &str,
        fragment_source: &str,
    ) -> Result<WebGlProgram, JsValue> {
        let vertex_shader = Self::compile_shader(gl, WebGl2RenderingContext::VERTEX_SHADER, vertex_source)?;
        let fragment_shader = Self::compile_shader(gl, WebGl2RenderingContext::FRAGMENT_SHADER, fragment_source)?;
        
        let program = gl.create_program().ok_or("Failed to create program")?;
        gl.attach_shader(&program, &vertex_shader);
        gl.attach_shader(&program, &fragment_shader);
        gl.link_program(&program);
        
        if gl.get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(program)
        } else {
            Err(JsValue::from_str(&format!(
                "Failed to link program: {}",
                gl.get_program_info_log(&program)
                    .unwrap_or_else(|| "Unknown error".into())
            )))
        }
    }
    
    // シェーダーをコンパイル
    fn compile_shader(
        gl: &WebGl2RenderingContext,
        shader_type: u32,
        source: &str,
    ) -> Result<WebGlShader, JsValue> {
        let shader = gl.create_shader(shader_type).ok_or("Failed to create shader")?;
        gl.shader_source(&shader, source);
        gl.compile_shader(&shader);
        
        if gl.get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(shader)
        } else {
            Err(JsValue::from_str(&format!(
                "Failed to compile shader: {}",
                gl.get_shader_info_log(&shader)
                    .unwrap_or_else(|| "Unknown error".into())
            )))
        }
    }
}
