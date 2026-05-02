use crate::reactive::signals::Signal;
use crate::state::{FrameworkState, NodeId, UiArena};
use crate::widgets::base::{BuildContext, RenderOutput, Widget};
use glam::Vec2;
use rore_layout::{LayoutEngine, Node as TaffyNode};
use rore_types::{FlexDirection, Style, Val}; // FlexDirection va Val qo'shildi

pub struct ForList<T: Clone + PartialEq + 'static> {
    pub items: Signal<Vec<T>>,
    pub builder: Box<dyn FnMut(T) -> Box<dyn Widget> + Send>,
    pub style: Style, // YANGI: O'z uslubiga ega bo'lishi kerak
    pub current_items: Vec<T>,
    pub child_nodes: Vec<NodeId>,
    pub taffy_node: Option<TaffyNode>,
    my_id: Option<NodeId>,
}

impl<T: Clone + PartialEq + 'static> ForList<T> {
    pub fn new<F>(items: Signal<Vec<T>>, builder: F) -> Self
    where
        F: FnMut(T) -> Box<dyn Widget> + Send + 'static,
    {
        // Ro'yxatlar asosan vertikal bo'ladi, shuning uchun default Column beramiz
        let mut default_style = Style::default();
        default_style.flex_direction = FlexDirection::Column;
        default_style.width = Val::Percent(100.0);

        Self {
            items,
            builder: Box::new(builder),
            style: default_style, // Uslubni ulaymiz
            current_items: Vec::new(),
            child_nodes: Vec::new(),
            taffy_node: None,
            my_id: None,
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl<T: Clone + PartialEq + Send + 'static> Widget for ForList<T> {
    // ... type_name qismi o'zgarishsiz ...
    fn type_name(&self) -> &'static str {
        "ForList"
    }

    fn build(
        mut self: Box<Self>,
        arena: &mut UiArena,
        engine: &mut LayoutEngine,
        ctx: &BuildContext,
    ) -> NodeId {
        let mut t_children = Vec::new();
        let initial_items = self.items.get_untracked();
        self.current_items = initial_items.clone();

        for item in initial_items {
            let child_widget = (self.builder)(item);
            let (_, child_id) =
                crate::reactive::signals::create_scope(|| child_widget.build(arena, engine, ctx));
            self.child_nodes.push(child_id);
            if let Some(&t_node) = arena.taffy_map.get(&child_id) {
                t_children.push(t_node);
            }
        }

        // XATO TUZATILDI: Endi Style::default() emas, self.style.clone() ishlatiladi!
        let taffy_node = engine.new_node(self.style.clone(), &t_children);

        let my_id = arena.allocate_node();
        self.my_id = Some(my_id);
        self.taffy_node = Some(taffy_node);
        arena.taffy_map.insert(my_id, taffy_node);
        arena.node_map.insert(taffy_node, my_id);

        let sig = self.items;
        crate::reactive::signals::create_effect(move || {
            let _ = sig.get();
            crate::reactive::command::CommandQueue::send(
                crate::reactive::command::UICommand::RebuildNode(my_id, 2),
            );
        });

        arena.widgets[my_id.0 as usize] = Some(self);
        my_id
    }

    // ... faylning qolgan qismi (rebuild va render) o'zgarishsiz qolaveradi ...

    // =========================================================================
    // INQILOB 2: O(N) Smart Diffing (Solishtirish) algoritmi
    // =========================================================================
    fn rebuild(&mut self, state: &mut FrameworkState, engine: &mut LayoutEngine, _action: u32) {
        let new_items = self.items.get_untracked();

        let mut prefix = 0;
        let mut suffix = 0;
        let old_len = self.current_items.len();
        let new_len = new_items.len();

        // 1. Oldindan o'xshashlarni topish
        while prefix < old_len
            && prefix < new_len
            && self.current_items[prefix] == new_items[prefix]
        {
            prefix += 1;
        }

        // 2. Orqadan o'xshashlarni topish
        while suffix < (old_len - prefix)
            && suffix < (new_len - prefix)
            && self.current_items[old_len - 1 - suffix] == new_items[new_len - 1 - suffix]
        {
            suffix += 1;
        }

        let old_end = old_len - suffix;
        let new_end = new_len - suffix;

        // 3. O'rtadagi keraksizlarni o'lim navbatiga (Drop Queue) qo'shish
        for i in prefix..old_end {
            let old_id = self.child_nodes[i];
            state.drop_queue.borrow_mut().push(old_id);
            // Dvigatel buni o'chirganda, Scope tizimi avtomat xotirani tozalaydi!
        }

        // 4. O'rtadagi YANgI elementlarni qurish
        let mut new_middle_nodes = Vec::new();
        for i in prefix..new_end {
            let item = new_items[i].clone();
            let child_widget = (self.builder)(item);
            let ctx = BuildContext {};

            // Yangi elementlarni ham toza Scope bilan quramiz
            let (_, child_id) = crate::reactive::signals::create_scope(|| {
                child_widget.build(&mut state.arena, engine, &ctx)
            });
            new_middle_nodes.push(child_id);
        }

        // 5. Node'larni yangi tartibda birlashtirish
        let mut next_child_nodes = Vec::with_capacity(new_len);
        next_child_nodes.extend_from_slice(&self.child_nodes[..prefix]);
        next_child_nodes.extend(new_middle_nodes);
        next_child_nodes.extend_from_slice(&self.child_nodes[old_end..]);

        self.child_nodes = next_child_nodes;
        self.current_items = new_items;

        // 6. Fizik daraxtni (Taffy) 1 marta yangilash
        if let Some(parent_taffy) = self.taffy_node {
            let mut taffy_children = Vec::with_capacity(self.child_nodes.len());
            for &child_id in &self.child_nodes {
                if let Some(&t_node) = state.arena.taffy_map.get(&child_id) {
                    taffy_children.push(t_node);
                }
            }
            let _ = engine.taffy.set_children(parent_taffy, &taffy_children);
            let _ = engine.taffy.dirty(parent_taffy);
        }
    }

    fn render(
        &self,
        engine: &LayoutEngine,
        state: &mut FrameworkState,
        taffy_node: TaffyNode,
        parent_pos: Vec2,
        clip_rect: Option<[f32; 4]>,
        path: String,
    ) -> RenderOutput {
        let mut output = RenderOutput::new();
        let layout = engine.get_final_layout(taffy_node, parent_pos.x, parent_pos.y);

        for (i, &child_id) in self.child_nodes.iter().enumerate() {
            if let Some(widget_ref) = state.arena.widgets[child_id.0 as usize].take() {
                if let Some(&child_node) = state.arena.taffy_map.get(&child_id) {
                    let child_output = widget_ref.render(
                        engine,
                        state,
                        child_node,
                        Vec2::new(layout.x, layout.y),
                        clip_rect,
                        format!("{}_for_{}", path, i),
                    );
                    output.extend(child_output);
                }
                state.arena.widgets[child_id.0 as usize] = Some(widget_ref);
            }
        }
        output
    }
}
