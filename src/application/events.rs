pub struct Listener<T> {
    callbacks: Vec<Box<dyn FnMut(&T)>>,
}

impl<T> Listener<T> {
    pub fn default() -> Self {
        Listener {
            callbacks: Vec::new(),
        }
    }

    pub fn add<C>(&mut self, callback: C)
    where
        C: FnMut(&T) + 'static,
    {
        self.callbacks.push(Box::new(callback));
    }

    pub fn remove<C>(&mut self, callback: C)
    where
        C: FnMut(&T) + 'static,
    {
        self.callbacks.retain(|element| {
            let raw_element: *const (dyn FnMut(&T)) = element.as_ref();
            let raw_callback: *const (dyn FnMut(&T)) = &callback;
            !std::ptr::eq(raw_element, raw_callback)
        });
    }

    pub fn emit(&mut self, args: T) {
        for callback in &mut self.callbacks {
            callback(&args);
        }
    }
}

pub struct Listeners {
    pub draw: Listener<()>,
    pub resize: Listener<(u32, u32)>,
}

impl Listeners {
    pub fn default() -> Self {
        Listeners {
            draw: Listener::default(),
            resize: Listener::default(),
        }
    }
}
