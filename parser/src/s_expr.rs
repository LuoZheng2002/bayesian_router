#[derive(Debug, Clone)]
pub enum SExpr {
    Atom(String),
    List(Vec<SExpr>),
}

impl SExpr {
    pub fn is_atom(&self) -> bool {
        matches!(self, SExpr::Atom(_))
    }

    pub fn is_list(&self) -> bool {
        matches!(self, SExpr::List(_))
    }

    pub fn as_atom(&self) -> Option<&String> {
        if let SExpr::Atom(atom) = self {
            Some(atom)
        } else {
            None
        }
    }

    pub fn as_list(&self) -> Option<&Vec<SExpr>> {
        if let SExpr::List(list) = self {
            Some(list)
        } else {
            None
        }
    }
}
