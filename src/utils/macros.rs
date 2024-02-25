// This macro takes a walkable buffer type (See WalkingVec)
// and a number type.
// The generated code walk for the exact size of the number type
// and then tries to convert the collected bytes to the number type.
macro_rules! walk_to_number {
    ( $array:expr, $type:ty ) => {
        <$type>::from_le_bytes(
            $array
                .walk(std::mem::size_of::<$type>())
                .try_into()
                .unwrap(),
        )
    };
}

pub(crate) use walk_to_number;
