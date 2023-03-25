use core::marker::PhantomData;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Rope<'a, T> {
    anchor: *mut T,
    lead: *mut T,
    phantom: PhantomData<&'a mut T>,
}

#[derive(Debug)]
pub enum Simul<'a, T> {
    Advance(&'a mut T),
    Hold(&'a mut T),
}

impl<'a, T> Rope<'a, T> {
    pub fn new(anchor: &'a mut T) -> Self {
        Self {
            anchor,
            lead: anchor,
            phantom: PhantomData,
        }
    }

    pub fn advance(&mut self, f: impl FnOnce(&'a mut T) -> &'a mut T) {
        // Soundness: The question of soundness here is quite pivotal, and somewhat dubious, but
        // essential. Morally, this function is called with exclusive access to self and by
        // extension the lead reference it contains, so it should be safe enough to allow a closure
        // to access that reference, mutate its contents, and return the same reference or one with
        // the same lifetime. As f must produce a mutable reference of the same lifetime, there is
        // no chance of the reference safely escaping through the closure.

        // The lifetime of the reference to self extends throughout the function. We obtain the
        // value of lead as a raw pointer safely; the actual unsafe code does not interact with
        // self. Since we have an exclusive reference to self, f cannot safely touch self either.
        // Since lead is a raw pointer, there is no actual other reference that points to the data.
        // Some methods can return a reference to it, but this reference is itself connected with
        // the lifetime of self, so our exclusive reference to self means that none of those can
        // be live within advance.
        self.lead = f(unsafe { &mut *self.lead });
    }

    pub fn anchor(&mut self) {
        self.anchor = self.lead;
    }

    pub fn fall(&mut self) {
        self.lead = self.anchor;
    }

    pub fn advance_simul(&mut self, f: impl FnOnce(&'a mut T) -> Simul<'a, T>) {
        // Soundness: see advance
        let old_lead: *mut T = self.lead;
        match f(unsafe { &mut *self.lead }) {
            Simul::Hold(new_lead) => self.lead = new_lead,
            Simul::Advance(new_lead) => {
                self.anchor = old_lead;
                self.lead = new_lead;
            }
        }
    }

    pub fn get_anchor(&self) -> *mut T {
        self.anchor
    }

    pub fn get_lead<'b, 'c: 'a + 'b>(&'b self) -> &'c T {
        unsafe { &*self.lead }
    }

    pub fn get_lead_mut<'b, 'c: 'a + 'b>(&'b mut self) -> &'c mut T {
        unsafe { &mut *self.lead }
    }

    pub fn into_lead(self) -> &'a mut T {
        unsafe { &mut *self.lead }
    }

    pub fn into_anchor(self) -> &'a mut T {
        // I think this should be sound, but it warrants a lot of thought. The main consideration in
        // recovering a reference to T (with lifetime at least 'a) from self.lead or self.anchor is
        // that an any step, the contents of self.lead and self.anchor were initialized from a
        // mutable reference to T with the same lifetime, and self also has that lifetime,
        // so we know that nothing else has been modifying them. lead could point somewhere inside
        // anchor, so if we recover anchor as a reference then we must make sure lead is
        // no longer accessible, or has been set to anchor.
        unsafe { &mut *self.anchor }
    }
}
