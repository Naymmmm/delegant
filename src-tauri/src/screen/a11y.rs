use std::error::Error;
use uiautomation::core::UIAutomation;

#[derive(Debug, Clone, serde::Serialize)]
pub struct A11yNode {
    pub id: i32,
    pub name: String,
    pub control_type: String,
    pub rect: (i32, i32, i32, i32), // x, y, width, height (or left, top, right, bottom)
}

pub fn get_a11y_tree() -> Result<Vec<A11yNode>, Box<dyn Error>> {
    let automation = UIAutomation::new()?;
    let walker = automation.get_control_view_walker()?;
    let root = automation.get_root_element()?;

    let mut nodes = Vec::new();
    let mut id_counter = 1;

    let mut stack = vec![root];
    let my_pid = std::process::id();

    while let Some(element) = stack.pop() {
        // Skip elements from our own application so the agent doesn't try to click itself
        if let Ok(pid) = element.get_process_id() {
            if pid as u32 == my_pid {
                continue;
            }
        }
        // Enqueue children (reverse order to process left-to-right on pop)
        let mut children = Vec::new();
        if let Ok(child) = walker.get_first_child(&element) {
            children.push(child.clone());
            let mut next = child;
            while let Ok(sibling) = walker.get_next_sibling(&next) {
                children.push(sibling.clone());
                next = sibling;
            }
        }
        for c in children.into_iter().rev() {
            stack.push(c);
        }

        // Process this element
        if let Ok(rect) = element.get_bounding_rectangle() {
            let width = rect.get_right() - rect.get_left();
            let height = rect.get_bottom() - rect.get_top();

            if width > 0 && height > 0 {
                if let Ok(offscreen) = element.is_offscreen() {
                    if !offscreen {
                        let name = element.get_name().unwrap_or_default();

                        // uiautomation exposes control_type as i32, or we can get localized control type
                        if let Ok(control_type_name) = element.get_localized_control_type() {
                            // Filter empty names and useless structure elements
                            if !name.trim().is_empty()
                                && (control_type_name.contains("button")
                                    || control_type_name.contains("link")
                                    || control_type_name.contains("edit")
                                    || control_type_name.contains("text")
                                    || control_type_name.contains("combo")
                                    || control_type_name.contains("check")
                                    || control_type_name.contains("list")
                                    || control_type_name.contains("tab")
                                    || control_type_name.contains("menu"))
                            {
                                nodes.push(A11yNode {
                                    id: id_counter,
                                    name: name.clone(),
                                    control_type: control_type_name,
                                    rect: (
                                        rect.get_left(),
                                        rect.get_top(),
                                        rect.get_right(),
                                        rect.get_bottom(),
                                    ),
                                });
                                id_counter += 1;
                            }
                        }
                    }
                }
            }
        }

        // Limit to prevent insane processing times on complex desktops
        if nodes.len() > 300 {
            break;
        }
    }

    Ok(nodes)
}
