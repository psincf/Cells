#[macro_export]
macro_rules! deref {
    ($item: ident, $field_name: tt, $item_type: ty) => {
        impl std::ops::Deref for $item {
            type Target = $item_type;
            fn deref(&self) -> &$item_type {
                &self.$field_name
            }
        }

        impl std::ops::DerefMut for $item {
            fn deref_mut(&mut self) -> &mut $item_type {
                &mut self.$field_name
            }
        }
    };
}

#[macro_export]
macro_rules! index {
    ($item: ident, $field_name: tt, $item_type: ty) => {
        impl std::ops::Index<usize> for $item {
            type Output = $item_type;
            fn index(&self, index: usize) -> &$item_type {
                &self.$field_name[index]
            }
        }

        impl std::ops::IndexMut<usize> for $item {
            fn index_mut(&mut self, index: usize) -> &mut $item_type {
                &mut self.$field_name[index]
            }
        }
    };
}

#[macro_export]
macro_rules! index_custom_index {
    ($item: ident, $index_type: ty, $field_name: tt, $item_type: ty) => {
        impl std::ops::Index<$index_type> for $item {
            type Output = $item_type;
            fn index(&self, index: $index_type) -> &$item_type {
                &self.$field_name[index]
            }
        }

        impl std::ops::IndexMut<$index_type> for $item {
            fn index_mut(&mut self, index: $index_type) -> &mut $item_type {
                &mut self.$field_name[index]
            }
        }
    };
}