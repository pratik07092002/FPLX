use crate::datamodels::api_response::ApiResponse;
pub fn success<T> (
    message : &str , 
    data: T,
) -> ApiResponse<T> {

    ApiResponse { success: true, status_code: 200,
         message: message.to_string(),
          data: Some(data) }
}

pub fn failure(
    message: &str,
    code: u16,
) -> ApiResponse<()> {
    ApiResponse {
        success: false,
        status_code: code,
        message: message.to_string(),
        data: None,
    }
}

