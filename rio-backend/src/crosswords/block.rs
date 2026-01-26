use crate::crosswords::pos::Line;

#[derive(Debug, Clone, PartialEq)]
pub enum BlockType {
    Prompt,
    Command,
    Output,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub block_type: BlockType,
    pub start_line: Line,
    pub end_line: Option<Line>,
    pub exit_code: Option<i32>,
}

impl Block {
    pub fn new(block_type: BlockType, start_line: Line) -> Self {
        Self {
            block_type,
            start_line,
            end_line: None,
            exit_code: None,
        }
    }
}
