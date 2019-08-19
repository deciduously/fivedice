// dom.rs contains macros from working with the DOM

macro_rules! append_attrs {
    ($document:ident, $el:ident, $( $attr:expr ),* ) => {
        $(
            let attr = $document.create_attribute($attr.0).expect("Could not instantiate DOM attribute");
            attr.set_value($attr.1);
            $el.set_attribute_node(&attr).expect("Could not set attribute");
        )*
    }
}

macro_rules! append_text_child {
    ($document:ident, $el:ident, $text:expr ) => {
        let text = $document.create_text_node($text);
        $el.append_child(&text)
            .expect("Could not append text to parent");
    };
}

macro_rules! create_element_attrs {
    ($document:ident, $type:expr, $( $attr:expr ),* ) => {
        {
        #[allow(clippy::let_and_return)]
        let el = $document.create_element($type).expect("Could not create element");
        append_attrs!($document, el, $( $attr ),*);
        el}
    }
}

macro_rules! append_element_attrs {
    ($document:ident, $parent:ident, $type:expr, $( $attr:expr ),* ) => {
        let el = create_element_attrs!($document, $type, $( $attr ),* );
        $parent.append_child(&el).expect("Could not append element to parent");
    }
}

macro_rules! append_text_element_attrs {
    ($document:ident, $parent:ident, $type:expr, $text:expr, $( $attr:expr ),*) => {
        let el = create_element_attrs!($document, $type, $( $attr ),* );
        append_text_child!($document, el, $text);
        $parent.append_child(&el).expect("Could not append text child to parent");
    }
}
