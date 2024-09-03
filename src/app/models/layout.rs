#[derive(Debug, Clone, Copy)]
pub enum LayoutType {
    L2x2,
    L2x3,
    L2x4,
    L3x2,
    L3x3,
    L3x4,
    L4x2,
    L4x3,
    L4x4,
    L4x5,
    L5x4,
    L5x5,
    L6x6,
    L8x8,
    L10x10,
}

impl LayoutType {
    pub fn value(&self) -> (usize, usize) {
        match &self {
            Self::L2x2 => (2, 2),
            Self::L2x3 => (2, 3),
            Self::L2x4 => (2, 4),
            Self::L3x2 => (3, 2),
            Self::L3x3 => (3, 3),
            Self::L3x4 => (3, 4),
            Self::L4x2 => (4, 2),
            Self::L4x3 => (4, 3),
            Self::L4x4 => (4, 4),
            Self::L4x5 => (4, 5),
            Self::L5x4 => (5, 4),
            Self::L5x5 => (5, 5),
            Self::L6x6 => (6, 6),
            Self::L8x8 => (8, 8),
            Self::L10x10 => (10, 10),
        }
    }
}

impl std::fmt::Display for LayoutType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "L{}x{}", self.value().0, self.value().1)
    }
}

#[derive(Debug, Clone)]
pub struct Layout {
    pub resource: String,
    pub layout_type: LayoutType,
}

impl Layout {
    pub fn new(resource: String, layout_type: LayoutType) -> Self {
        Self {
            resource,
            layout_type,
        }
    }
}
