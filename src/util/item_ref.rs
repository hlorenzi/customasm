pub struct ItemRef<T>(
    pub usize,
    std::marker::PhantomData<*const T>);


impl<T> ItemRef<T>
{
    pub fn new(value: usize) -> Self
    {
        ItemRef(value, std::marker::PhantomData)
    }
}


impl<T> Clone for ItemRef<T>
{
    fn clone(&self) -> Self
    {
        ItemRef(self.0, std::marker::PhantomData)
    }
}


impl<T> Copy for ItemRef<T> {}


impl<T> PartialEq for ItemRef<T>
{
    fn eq(&self, other: &Self) -> bool
    {
        self.0 == other.0
    }
}


impl<T> std::fmt::Debug for ItemRef<T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let name = std::any::type_name::<T>();

        f.write_str("ItemRef<")?;
        f.write_str(name.rsplit_once("::").map(|n| n.1).unwrap_or(name))?;
        f.write_str(">(")?;
        self.0.fmt(f)?;
        f.write_str(")")?;
        Ok(())
    }
}