use crate::player::Instruction;
use crate::remote::fns::Instruct;
use itertools::Itertools;
use leptos::*;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[component]
pub fn Browse(
    cx: Scope,
    media_files: Vec<String>
) -> impl IntoView {
    let (browser, set_browser) = create_signal(cx, Browser::build(media_files));

    let instruct = create_server_action::<Instruct>(cx);

    move || { 
        browser.with(|browser| {
            let focus = browser.focus.as_ref().borrow();

            view! { cx,
                <div class="card shadow mt-5">
                    <div class="card-body">
                        <div class="mb-3">
                            {browser
                                .build_breadcrumb()
                                .into_iter()
                                .map(|(name, dir)| {
                                    view! { cx,
                                        <button
                                            class="btn btn-link"
                                            on:click=move |_| {
                                                set_browser.update(|b| { b.set_focus(&dir) });
                                            }
                                        >
                                            " > "{name}
                                        </button>
                                    }.into_any()
                                })
                                .collect::<Vec<_>>()
                            }
                        </div>
                        <div class="row">
                            {focus.children
                                .iter()
                                .map(|dir| {
                                    let d = dir.clone();
                                    let name = dir.as_ref().borrow().name.clone();

                                    view! { cx,
                                        <div class="col-12 col-md-4 col-lg-3 d-grid">
                                            <button
                                                class="btn btn-link text-decoration-none p-0"
                                                on:click=move |_| {
                                                    set_browser.update(|b| { b.set_focus(&d) });
                                                }
                                            >
                                                <div class="alert alert-light fs-4">
                                                    <i class="bx bx-folder me-3"></i>
                                                    {name}
                                                </div>
                                            </button>
                                        </div>
                                    }
                                })
                                .collect::<Vec<_>>()
                            }
                        </div>
                        <hr/>
                        <div class="row">
                            {focus.files
                                .iter()
                                .map(|file| {
                                    let path = file.path.clone();
                                    view! { cx,
                                        <div class="col-12 col-md-4 col-lg-3 d-grid">
                                            <button
                                                class="btn btn-link text-decoration-none p-0"
                                                on:click=move |_| {
                                                    let i = Instruct {
                                                        i: Instruction::Play(path.clone(), 0.0)
                                                    };
                                                    instruct.dispatch(i);
                                                }
                                            >
                                                <div class="alert alert-light fs-5">
                                                    <i class="bx bx-file me-3"></i>
                                                    {file.name.clone()}
                                                </div>
                                            </button>
                                        </div>
                                    }
                                })
                                .collect::<Vec<_>>()
                            }
                        </div>
                    </div>
                </div>
            }
        })
    }
}

type DirRef = Rc<RefCell<Dir>>;

#[derive(Clone)]
struct Browser {
    // Needed to avoid root ref from being dropped since only referenced by weak `parent` field
    #[allow(dead_code)]
    root: DirRef,
    focus: DirRef
}

#[derive(Clone, Debug)]
struct Dir {
    name: String,
    parent: Weak<RefCell<Dir>>,
    children: Vec<DirRef>,
    files: Vec<MediaFile>
}

#[derive(Clone, Debug)]
struct MediaFile {
    path: String,
    name: String
}

impl Browser {
    fn build(paths: Vec<String>) -> Self {
        let root = Rc::new(
            RefCell::new(Dir {
                name: "Home".to_string(),
                parent: Weak::new(),
                children: vec![],
                files: Self::files_with_prefix("", &paths)
            })
        );

        let focus = root.clone();

        root.borrow_mut().children = Self::dirs_with_prefix(&root, "", &paths);

        Self {
            root,
            focus
        }
    }

    fn build_breadcrumb(&self) -> Vec<(String, DirRef)> {
        let focus = self.focus.clone();
        let focus_ref = focus.as_ref().borrow();

        focus_ref.build_breadcrumb(self.focus.clone())
    }

    fn set_focus<'a>(&mut self, focus: &'a DirRef) {
        self.focus = focus.clone();
    }

    fn files_with_prefix(prefix: &str, paths: &Vec<String>) -> Vec<MediaFile> {
        let mut files = paths.iter()
            .filter_map(|path| {
                path.strip_prefix(prefix).map(|suffix| (path, suffix))
            })
            .filter(|(_, suffix)| suffix.find("/").is_none())
            .map(|(path, suffix)| {
                MediaFile {
                    path: path.to_string(),
                    name: suffix.to_string()
                }
            })
            .collect::<Vec<_>>();

        files.sort_by(|a, b| a.name.cmp(&b.name));

        files
    }

    fn dirs_with_prefix(parent: &DirRef, prefix: &str, paths: &Vec<String>) -> Vec<DirRef> {
        let mut dirs = paths.iter()
            .filter_map(|path| {
                path.strip_prefix(prefix)
                    .and_then(|suffix| suffix.find("/").map(|ix| suffix.split_at(ix).0))
            })
            .unique()
            .map(|name| {
                let new_prefix = format!("{}{}/", prefix, name);

                let dir = Rc::new(
                    RefCell::new(Dir {
                        name: name.to_string(),
                        parent: Rc::downgrade(parent),
                        children: vec![],
                        files: Self::files_with_prefix(&new_prefix, paths)
                    })
                );

                dir.borrow_mut().children = Self::dirs_with_prefix(&dir, &new_prefix, paths);
                dir
            })
            .collect::<Vec<_>>();

        dirs.sort_by(|a, b| {
            a.as_ref().borrow().name.cmp(&b.as_ref().borrow().name)
        });

        dirs
    }
}

impl Dir {
    fn build_breadcrumb(&self, self_ref: DirRef) -> Vec<(String, DirRef)> {
        let mut items = vec![];

        if let Some(parent) = self.parent.upgrade() {
            let mut parent_breadcrumb = parent.as_ref().borrow().build_breadcrumb(parent.clone());
            items.append(&mut parent_breadcrumb);
        }

        items.push((self.name.clone(), self_ref));
        items
    }
}
