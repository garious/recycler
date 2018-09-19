use std::sync::Mutex;

/// A function that leaves the given type in the same state as Default,
/// but starts with an existing type instead of allocating a new one.
pub trait Reset {
    fn reset(&mut self);
}

/// An value that's returned to its heap once dropped.
pub struct Recyclable<'a, T: 'a + Default + Reset> {
    val: Option<T>,
    landfill: &'a Mutex<Vec<T>>,
}

impl<'a, T: Default + Reset> AsRef<T> for Recyclable<'a, T> {
    fn as_ref(&self) -> &T {
        self.val.as_ref().unwrap()
    }
}

impl<'a, T: Default + Reset> AsMut<T> for Recyclable<'a, T> {
    fn as_mut(&mut self) -> &mut T {
        self.val.as_mut().unwrap()
    }
}

impl<'a, T: Default + Reset> Drop for Recyclable<'a, T> {
    fn drop(&mut self) {
        if let Some(val) = self.val.take() {
            self.landfill.lock().unwrap().push(val);
        }
    }
}

/// An object to minimize memory allocations. Use `allocate()`
/// to get recyclable values of type `T`. When those recyclables
/// are dropped, they're returned to the recycler. The next time
/// `allocate()` is called, the value will be pulled from the
/// recycler instead being allocated from memory.
pub struct Recycler<T: Default + Reset> {
    landfill: Mutex<Vec<T>>,
}

impl<T: Default + Reset> Default for Recycler<T> {
    fn default() -> Self {
        Recycler {
            landfill: Mutex::new(vec![]),
        }
    }
}

impl<T: Default + Reset> Recycler<T> {
    pub fn allocate(&self) -> Recyclable<T> {
        let val = self
            .landfill
            .lock()
            .unwrap()
            .pop()
            .map(|mut val| {
                val.reset();
                val
            }).unwrap_or_default();
        Recyclable {
            val: Some(val),
            landfill: &self.landfill,
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate crossbeam;

    use super::*;
    use std::sync::mpsc::channel;

    #[derive(Default)]
    struct Foo {
        x: u8,
    }

    impl Reset for Foo {
        fn reset(&mut self) {
            self.x = 0;
        }
    }

    #[test]
    fn test_allocate() {
        let recycler: Recycler<Foo> = Recycler::default();
        assert_eq!(recycler.allocate().as_ref().x, 0);
    }

    #[test]
    fn test_recycle() {
        let recycler: Recycler<Foo> = Recycler::default();

        {
            let mut foo = recycler.allocate();
            foo.as_mut().x = 1;
        }
        assert_eq!(recycler.landfill.lock().unwrap().len(), 1);

        let foo = recycler.allocate();
        assert_eq!(foo.as_ref().x, 0);
        assert_eq!(recycler.landfill.lock().unwrap().len(), 0);
    }

    #[test]
    fn test_channel() {
        let recycler: Recycler<Foo> = Recycler::default();
        let (sender, receiver) = channel();
        {
            let mut foo = recycler.allocate();
            foo.as_mut().x = 1;
            sender.send(foo).unwrap();
            assert_eq!(recycler.landfill.lock().unwrap().len(), 0);
        }
        {
            let foo = receiver.recv().unwrap();
            assert_eq!(foo.as_ref().x, 1);
            assert_eq!(recycler.landfill.lock().unwrap().len(), 0);
        }
        assert_eq!(recycler.landfill.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_scoped_thread() {
        let recycler: Recycler<Foo> = Recycler::default();
        let (sender, receiver) = channel();
        sender.send(recycler.allocate()).unwrap();

        crossbeam::scope(|scope| {
            scope.spawn(move || {
                receiver.recv().unwrap();
            });
        });

        assert_eq!(recycler.landfill.lock().unwrap().len(), 1);
    }

    struct ThreadNanny<'a> {
        _hdl: crossbeam::thread::ScopedJoinHandle<'a, ()>,
    }

    #[test]
    fn test_thread_lifetime_in_struct() {
        let recycler: Recycler<Foo> = Recycler::default();
        let (sender, receiver) = channel();
        sender.send(recycler.allocate()).unwrap();

        {
            let _hdl = crossbeam::scope(|scope| {
                scope.spawn(move || {
                    receiver.recv().unwrap();
                })
            });
            ThreadNanny { _hdl };
        }

        assert_eq!(recycler.landfill.lock().unwrap().len(), 1);
    }
}
