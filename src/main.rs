use inkwell::{
    context::Context,
    module::Linkage,
    AddressSpace
};

fn main() {
    let context = Context::create();
    let module = context.create_module("main");
    let builder = context.create_builder();
    let i32_type = context.i32_type();
    let i32_ptr_type = i32_type.ptr_type(AddressSpace::Generic);
    let i8_type = context.i8_type();
    let i8_ptr_type = i8_type.ptr_type(AddressSpace::Generic);

    // @HELLO_WORLD = private unnamed_addr constant [13 x i8] c"Hello, world!", align 1
    let hello_world = context.const_string(b"Hello, world!", false);
    let global = module.add_global(hello_world.get_type(), None, "HELLO_WORLD");
    global.set_linkage(Linkage::Private);
    global.set_unnamed_addr(true);
    global.set_constant(true);
    global.set_initializer(&hello_world);
    global.set_alignment(1);

    // HANDLE WINAPI GetStdHandle(
    //     _In_ DWORD nStdHandle
    // );
    // declare i8* @GetStdHandle(i32)
    let getstdhandle_type = i8_ptr_type.fn_type(&[i32_type.into()], false);
    module.add_function("GetStdHandle", getstdhandle_type, None);

    // BOOL WINAPI WriteConsoleA(
    //     _In_ HANDLE hConsoleOutput,
    //     _In_ const VOID *lpBuffer,
    //     _In_ DWORD nNumberOfCharsToWrite,
    //     _Out_ LPDWORD lpNumberOfCharsWritten,
    //     _Reserved_ LPVOID lpReserved
    // )
    // declare i8 @WriteConsoleA(i8*, i8*, i32, i32*, i8*)
    let writeconsole_type = i8_type.fn_type(&[
        i8_ptr_type.into(),
        i8_ptr_type.into(),
        i32_type.into(),
        i32_ptr_type.into(),
        i8_ptr_type.into()], false);
    module.add_function("WriteConsoleA", writeconsole_type, None);

    // define i32 @main()
    let main_type = i32_type.fn_type(&[], false);
    let function = module.add_function("main", main_type, None);
    let basic_block = context.append_basic_block(function, "entry");
    builder.position_at_end(&basic_block);

    // call i8* @GetStdHandle(i32 -11)
    let fun = module.get_function("GetStdHandle");
    let handle = builder.build_call(
        fun.unwrap(),
        &[i32_type.const_int(0xFFFF_FFF5, false).into()],
        "GetStdHandle"
    );

    // call i8 @WriteConsoleA(
    //     i8* %GetStdHandle,
    //     i8* getelementptr inbounds ([13 x i8], [13 x i8]*, @HelloWorld, i32 0, i32 0),
    //     i32 13,
    //     i32* null,
    //     i8* null
    // )
    let fun = module.get_function("WriteConsoleA");
    let global_value = module.get_global("HELLO_WORLD");
    builder.build_call(
        fun.unwrap(),
        &[
            handle.try_as_basic_value().left().unwrap(),
            global_value.unwrap().as_pointer_value().const_cast(i8_ptr_type).into(),
            i32_type.const_int(global_value.unwrap().get_initializer().unwrap().into_array_value().get_type().len() as u64, false).into(),
            i32_ptr_type.const_null().into(),
            i8_ptr_type.const_null().into()
        ],
        "WriteConsoleA"
    );

    // ret i32 0
    builder.build_return(Some(&i32_type.const_int(0, false)));

    if !module.write_bitcode_to_path("main.bc".as_ref()) {
        panic!("failed create file");
    }
}
