#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointRole {
    Client,
    Server,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RequestIdAllocator {
    next: Option<u64>,
}

impl RequestIdAllocator {
    pub fn new(role: EndpointRole) -> Self {
        let first = match role {
            EndpointRole::Client => 0,
            EndpointRole::Server => 1,
        };

        Self { next: Some(first) }
    }

    pub fn allocate(&mut self) -> Option<u64> {
        let current = self.next?;
        self.next = current.checked_add(2);
        Some(current)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RequestIdValidationError {
    IncorrectParity {
        request_id: u64,
        sender: EndpointRole,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct RequestIdValidator {
    sender: EndpointRole,
}

impl RequestIdValidator {
    pub fn new(sender: EndpointRole) -> Self {
        Self { sender }
    }

    pub fn validate(&mut self, request_id: u64) -> Result<(), RequestIdValidationError> {
        let expected_lsb = match self.sender {
            EndpointRole::Client => 0,
            EndpointRole::Server => 1,
        };

        if request_id & 1 != expected_lsb {
            return Err(RequestIdValidationError::IncorrectParity {
                request_id,
                sender: self.sender,
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{EndpointRole, RequestIdAllocator, RequestIdValidationError, RequestIdValidator};

    #[test]
    fn client_starts_with_zero() {
        let mut allocator = RequestIdAllocator::new(EndpointRole::Client);

        assert_eq!(allocator.allocate(), Some(0));
    }

    #[test]
    fn client_increments_by_two() {
        let mut allocator = RequestIdAllocator::new(EndpointRole::Client);

        assert_eq!(allocator.allocate(), Some(0));
        assert_eq!(allocator.allocate(), Some(2));
        assert_eq!(allocator.allocate(), Some(4));
    }

    #[test]
    fn server_starts_with_one_and_increments_by_two() {
        let mut allocator = RequestIdAllocator::new(EndpointRole::Server);

        assert_eq!(allocator.allocate(), Some(1));
        assert_eq!(allocator.allocate(), Some(3));
        assert_eq!(allocator.allocate(), Some(5));
    }

    #[test]
    fn returns_none_after_allocating_the_last_client_id() {
        let mut allocator = RequestIdAllocator {
            next: Some(u64::MAX - 1),
        };

        assert_eq!(allocator.allocate(), Some(u64::MAX - 1));
        assert_eq!(allocator.allocate(), None);
    }

    #[test]
    fn accepts_even_request_id_from_client() {
        let mut validator = RequestIdValidator::new(EndpointRole::Client);

        assert_eq!(validator.validate(0), Ok(()));
    }

    #[test]
    fn rejects_odd_request_id_from_client() {
        let mut validator = RequestIdValidator::new(EndpointRole::Client);

        assert_eq!(
            validator.validate(1),
            Err(RequestIdValidationError::IncorrectParity {
                request_id: 1,
                sender: EndpointRole::Client,
            }),
        );
    }
}
