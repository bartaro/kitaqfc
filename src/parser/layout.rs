use super::*;
impl Parser {
    fn storage_size(&self, ty: &CType) -> i32 {
        match &ty.kind {
            TypeKind::Simple(Simple::UInt8 | Simple::Int8) | TypeKind::Enum(_) => 1,
            TypeKind::Simple(Simple::UInt16 | Simple::Int16) | TypeKind::Pointer(_) => 2,
            TypeKind::Array(t, n) => self.storage_size(t) * (*n).max(0),
            TypeKind::ArrayExpression(t, e) if e.is(t::INTEGER) => {
                self.storage_size(t) * e.int(1).unwrap_or(0).max(0)
            }
            TypeKind::Struct(n) | TypeKind::Union(n) => {
                self.aggregates.get(n).map_or(0, |a| a.size.max(0))
            }
            _ => 0,
        }
    }
    pub(super) fn layout_fields(
        &self,
        mut fields: Vec<Field>,
        is_union: bool,
        packed: bool,
        forced_align: i32,
    ) -> (Vec<Field>, i32, i32) {
        let (mut offset, mut natural, mut maximum) = (0, 1, 0);
        for field in &mut fields {
            let size = self.storage_size(&field.ty);
            let align = if packed {
                1
            } else {
                match &field.ty.kind {
                    TypeKind::Struct(n) | TypeKind::Union(n) => {
                        self.aggregates.get(n).map_or(1, |a| a.alignment.max(1))
                    }
                    _ => {
                        if size >= 2 {
                            2
                        } else {
                            1
                        }
                    }
                }
            };
            natural = natural.max(align);
            maximum = maximum.max(size);
            field.offset = if is_union {
                0
            } else {
                offset = (offset + align - 1) & !(align - 1);
                let value = offset;
                offset += size;
                value
            };
        }
        let alignment = if forced_align > 0 {
            forced_align
        } else {
            natural
        };
        let mut size = if is_union { maximum } else { offset };
        if !packed {
            size = (size + alignment - 1) & !(alignment - 1);
        }
        (fields, size, alignment)
    }
}
