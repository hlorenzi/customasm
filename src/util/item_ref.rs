#[derive(Copy, Clone, Eq, PartialEq)]
struct NonMaxUsize(std::num::NonZeroUsize);


impl NonMaxUsize
{
    pub fn new(value: usize) -> NonMaxUsize
    {
        NonMaxUsize(std::num::NonZeroUsize::new(!value).unwrap())
    }


    pub fn get(&self) -> usize
    {
        !self.0.get()
    }
}


pub struct ItemRef<T>(
    NonMaxUsize,
    std::marker::PhantomData<*const T>);


impl<T> ItemRef<T>
{
    pub fn new(value: usize) -> Self
    {
        ItemRef(NonMaxUsize::new(value), std::marker::PhantomData)
    }


    pub fn get_raw(&self) -> usize
    {
        self.0.get()
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
        self.get_raw().fmt(f)?;
        f.write_str(")")?;
        Ok(())
    }
}