mod event;

use event::Event;

/// Event queue element
struct EventNode {
    /// The related event
    pub event: Event,
    /// The next node in the queue
    pub next: Option<Box<EventNode>>,
    /// The next node in the stack
    pub next_in_stack: Option<Box<EventNode>>,
}

/// Event queue
pub struct EventQueue {
    /// Pointer to first element of the queue, if any
    head: Option<Box<EventNode>>
}

impl EventQueue {
    pub fn enqueue(&mut self, new_event: Event) {
        let mut current = &mut self.head;

        while current.as_ref().is_some_and(|node| node.event < new_event ) {
            current = &mut current.as_mut().unwrap().next;
        }

        match current.take() {
            Some(mut existing_node) => {
                if existing_node.event == new_event {
                    *current = Some(Box::new(EventNode {
                        event: new_event,
                        next: existing_node.next.take(),
                        next_in_stack: Some(existing_node)
                    }));
                } else {
                    *current = Some(Box::new(EventNode {
                        event: new_event,
                        next: Some(existing_node),
                        next_in_stack: None
                    }));
                }
            },
            None => *current = Some(Box::new(EventNode {
                    event: new_event,
                    next: None,
                    next_in_stack: None
                }))
        }
    }

    pub fn pop(&mut self) -> Option<Event> {
        self.head.take().map(|mut node| {
            if let Some(mut stacked_node) = node.next_in_stack.take() {
                stacked_node.next = node.next.take();
                self.head = Some(stacked_node);
            } else {
                self.head = node.next.take();
            }
            node.event
        })
    }
}
