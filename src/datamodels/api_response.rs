use serde::Serialize;

#[derive(Debug,Serialize)]
pub struct ApiResponse<T> {
    pub success:bool ,
    pub status_code: u16,
    pub message: String,
    pub data : Option<T>
}