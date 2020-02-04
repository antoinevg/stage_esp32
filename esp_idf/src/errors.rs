use failure::Fail; // see: https://github.com/rust-lang-nursery/failure


#[derive(Debug, Fail)]
#[fail(display = "my error")]
struct MyError;

#[derive(Debug, Fail)]
#[fail(display = "my wrapping error")]
struct WrappingError(#[fail(cause)] MyError);

fn bad_function() -> Result<(), WrappingError> {
    Err(WrappingError(MyError))
}


#[derive(Fail, Debug)]
#[fail(display = "An error occurred with error code {}. ({})", code, message)]
struct MyOtherError {
    code: i32,
    message: str,
}
