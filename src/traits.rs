use input;
use gust::mesh;
use core::program;

#[derive(Debug)]
pub enum Error {
    Program(program::Error),
    Mesh(mesh::Error)
}

impl From<program::Error> for Error {
    fn from(other: program::Error) -> Self {
        Error::Program(other)
    }
}

impl From<mesh::Error> for Error {
    fn from(other: mesh::Error) -> Self {
        Error::Mesh(other)
    }
}

pub trait Emitting {
    fn shine(&self, input: &input::DrawInput) -> Result<(), Error>;
}

pub trait Reflecting {
    fn reflect(&self, input: &input::DrawInput) -> Result<(), Error>;
}