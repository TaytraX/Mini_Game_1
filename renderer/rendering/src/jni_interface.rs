use jni::JNIEnv;
use jni::objects::{JClass, JFloatArray};
use jni::sys::jfloatArray;
use std::sync::{RwLock, OnceLock, Mutex};

static GLOBAL_CHUNK_DATA: OnceLock<RwLock<Vec<f32>>> = OnceLock::new();
static CHUNK_UPDATED: OnceLock<Mutex<bool>> = OnceLock::new();

/// Initialiser le stockage global des données de chunk
fn get_chunk_storage() -> &'static RwLock<Vec<f32>> {
    GLOBAL_CHUNK_DATA.get_or_init(|| {
        // Initialiser avec un chunk vide de 32x32x32
        RwLock::new(vec![0.0; 32 * 32 * 32])
    })
}

/// Initialiser le flag de mise à jour
fn get_update_flag() -> &'static Mutex<bool> {
    CHUNK_UPDATED.get_or_init(|| Mutex::new(false))
}

/// Recevoir les données du chunk depuis Java
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_Teste_updateChunk<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    chunk_data: jfloatArray,
) {
    // Convertir le tableau Java en Vec<f32> Rust
    let java_array = unsafe { JFloatArray::from_raw(chunk_data) };

    match env.get_array_length(&java_array) {
        Ok(len) => {
            let mut rust_array = vec![0.0f32; len as usize];

            match env.get_float_array_region(&java_array, 0, &mut rust_array) {
                Ok(_) => {
                    // Mettre à jour le stockage global
                    if let Ok(mut storage) = get_chunk_storage().write() {
                        *storage = rust_array;

                        // Marquer qu'une mise à jour est nécessaire
                        if let Ok(mut flag) = get_update_flag().lock() {
                            *flag = true;
                        }

                        println!("Chunk data updated successfully: {} blocks", len);
                    } else {
                        eprintln!("Failed to acquire write lock on chunk storage");
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get float array region: {:?}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to get array length: {:?}", e);
        }
    }
}

/// Récupérer les données du chunk depuis le stockage global
pub fn get_chunk_data() -> Vec<f32> {
    get_chunk_storage()
        .read()
        .map(|data| data.clone())
        .unwrap_or_else(|_| vec![0.0; 32 * 32 * 32])
}

/// Vérifier si le chunk a été mis à jour et réinitialiser le flag
pub fn check_and_clear_update_flag() -> bool {
    get_update_flag()
        .lock()
        .map(|mut flag| {
            let updated = *flag;
            *flag = false;
            updated
        })
        .unwrap_or(false)
}