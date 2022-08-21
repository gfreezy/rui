use style::{
    axis::Axis,
    layout::{
        CrossAxisAlignment, MainAxisAlignment, MainAxisSize, TextDirection, VerticalDirection,
    },
};



pub(crate) struct RenderFlex {
    direction: Axis,
    main_axis_size: MainAxisSize,
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
    text_direction: TextDirection,
    vertical_direction: VerticalDirection,
}

impl RenderFlex {
    pub fn new(
        direction: Axis,
        main_axis_size: MainAxisSize,
        main_axis_alignment: MainAxisAlignment,
        cross_axis_alignment: CrossAxisAlignment,
        text_direction: TextDirection,
        vertical_direction: VerticalDirection,
    ) -> Self {
        let render = Self {
            direction,
            main_axis_size,
            main_axis_alignment,
            cross_axis_alignment,
            text_direction,
            vertical_direction,
        };
        render
    }

    pub fn set_main_axis_size(&mut self, main_axis_size: MainAxisSize) {
        self.main_axis_size = main_axis_size;
    }
}
