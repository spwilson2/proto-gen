//use crate::parser::Parser;
///// Repr of a fully parsed and validated proto.
//pub struct ParseTree {}
//
//pub struct MessageIter<'t> {
//    tree: &'t ParseTree,
//    idx: usize,
//}
//
//pub struct RpcIter<'t> {
//    tree: &'t ParseTree,
//    idx: usize,
//}
//
//pub struct ServiceIter<'t> {
//    tree: &'t ParseTree,
//    idx: usize,
//}
//
//impl ParseTree {
//    pub fn new<I>(parser: Parser<I>) -> Self
//    where
//        I: Iterator<Item = char>,
//    {
//        todo!();
//        Self {}
//    }
//    pub fn messages(&self) -> MessageIter<'_> {
//        todo!()
//    }
//    pub fn services(&self) -> RpcIter<'_> {
//        todo!()
//    }
//}
