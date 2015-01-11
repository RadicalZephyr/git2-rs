use std::marker;
use std::str;

use {raw, signature, Error, Oid, Object, Signature, ObjectType};
use util::Binding;

/// A structure to represent a git [tag][1]
///
/// [1]: http://git-scm.com/book/en/Git-Basics-Tagging
pub struct Tag<'repo> {
    raw: *mut raw::git_tag,
    marker: marker::ContravariantLifetime<'repo>,
}

impl<'repo> Tag<'repo> {
    /// Get the id (SHA1) of a repository tag
    pub fn id(&self) -> Oid {
        unsafe { Binding::from_raw(raw::git_tag_id(&*self.raw)) }
    }

    /// Get the message of a tag
    ///
    /// Returns None if there is no message or if it is not valid utf8
    pub fn message(&self) -> Option<&str> {
        self.message_bytes().and_then(|s| str::from_utf8(s).ok())
    }

    /// Get the message of a tag
    ///
    /// Returns None if there is no message
    pub fn message_bytes(&self) -> Option<&[u8]> {
        unsafe { ::opt_bytes(self, raw::git_tag_message(&*self.raw)) }
    }

    /// Get the name of a tag
    ///
    /// Returns None if it is not valid utf8
    pub fn name(&self) -> Option<&str> {
        str::from_utf8(self.name_bytes()).ok()
    }

    /// Get the name of a tag
    pub fn name_bytes(&self) -> &[u8] {
        unsafe { ::opt_bytes(self, raw::git_tag_name(&*self.raw)).unwrap() }
    }

    /// Recursively peel a tag until a non tag git_object is found
    pub fn peel(&self) -> Result<Object<'repo>, Error> {
        let mut ret = 0 as *mut raw::git_object;
        unsafe {
            try_call!(raw::git_tag_peel(&mut ret, &*self.raw));
            Ok(Binding::from_raw(ret))
        }
    }

    /// Get the tagger (author) of a tag
    ///
    /// If the author is unspecified, then `None` is returned.
    pub fn tagger(&self) -> Option<Signature> {
        unsafe {
            let ptr = raw::git_tag_tagger(&*self.raw);
            if ptr.is_null() {
                None
            } else {
                Some(signature::from_raw_const(self, ptr))
            }
        }
    }

    /// Get the tagged object of a tag
    ///
    /// This method performs a repository lookup for the given object and
    /// returns it
    pub fn target(&self) -> Result<Object<'repo>, Error> {
        let mut ret = 0 as *mut raw::git_object;
        unsafe {
            try_call!(raw::git_tag_target(&mut ret, &*self.raw));
            Ok(Binding::from_raw(ret))
        }
    }

    /// Get the OID of the tagged object of a tag
    pub fn target_id(&self) -> Oid {
        unsafe { Binding::from_raw(raw::git_tag_target_id(&*self.raw)) }
    }

    /// Get the OID of the tagged object of a tag
    pub fn target_type(&self) -> Option<ObjectType> {
        unsafe { ObjectType::from_raw(raw::git_tag_target_type(&*self.raw)) }
    }
}

impl<'repo> Binding for Tag<'repo> {
    type Raw = *mut raw::git_tag;
    unsafe fn from_raw(raw: *mut raw::git_tag) -> Tag<'repo> {
        Tag {
            raw: raw,
            marker: marker::ContravariantLifetime,
        }
    }
    fn raw(&self) -> *mut raw::git_tag { self.raw }
}

#[unsafe_destructor]
impl<'repo> Drop for Tag<'repo> {
    fn drop(&mut self) {
        unsafe { raw::git_tag_free(self.raw) }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let (_td, repo) = ::test::repo_init();
        let head = repo.head().unwrap();
        let id = head.target().unwrap();
        assert!(repo.find_tag(id).is_err());

        let obj = repo.find_object(id, None).unwrap();
        let sig = repo.signature().unwrap();
        let tag_id = repo.tag("foo", &obj, &sig, "msg", false).unwrap();
        let tag = repo.find_tag(tag_id).unwrap();
        assert_eq!(tag.id(), tag_id);

        let tags = repo.tag_names(None).unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags.get(0), Some("foo"));

        assert_eq!(tag.name(), Some("foo"));
        assert_eq!(tag.message(), Some("msg"));
        assert_eq!(tag.peel().unwrap().id(), obj.id());
        assert_eq!(tag.target_id(), obj.id());
        assert_eq!(tag.target_type(), Some(::ObjectType::Commit));

        assert_eq!(tag.tagger().unwrap().name(), sig.name());
        tag.target().unwrap();

        repo.tag_delete("foo").unwrap();
    }
}
