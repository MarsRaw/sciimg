; ModuleID = 'probe5.1cf41cb4-cgu.0'
source_filename = "probe5.1cf41cb4-cgu.0"
target datalayout = "e-m:o-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-apple-macosx10.7.0"

@alloc_a2f8eb6d3f9d0f7e3a48bd958d093d6c = private unnamed_addr constant <{ [75 x i8] }> <{ [75 x i8] c"/rustc/7b4f48927dce585f747a58083b45ab62b9d73a53/library/core/src/num/mod.rs" }>, align 1
@alloc_92d07dbbb76646aadf049f4cbee43205 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_a2f8eb6d3f9d0f7e3a48bd958d093d6c, [16 x i8] c"K\00\00\00\00\00\00\00/\04\00\00\05\00\00\00" }>, align 8
@str.0 = internal constant [25 x i8] c"attempt to divide by zero"

; probe5::probe
; Function Attrs: uwtable
define void @_ZN6probe55probe17he71ff36c39780cb5E() unnamed_addr #0 {
start:
  %0 = call i1 @llvm.expect.i1(i1 false, i1 false)
  br i1 %0, label %panic.i, label %"_ZN4core3num21_$LT$impl$u20$u32$GT$10div_euclid17h5ab8c74b8df8a2edE.exit"

panic.i:                                          ; preds = %start
; call core::panicking::panic
  call void @_ZN4core9panicking5panic17hd1729cd52f38b63cE(ptr align 1 @str.0, i64 25, ptr align 8 @alloc_92d07dbbb76646aadf049f4cbee43205) #3
  unreachable

"_ZN4core3num21_$LT$impl$u20$u32$GT$10div_euclid17h5ab8c74b8df8a2edE.exit": ; preds = %start
  ret void
}

; Function Attrs: nocallback nofree nosync nounwind readnone willreturn
declare i1 @llvm.expect.i1(i1, i1) #1

; core::panicking::panic
; Function Attrs: cold noinline noreturn uwtable
declare void @_ZN4core9panicking5panic17hd1729cd52f38b63cE(ptr align 1, i64, ptr align 8) unnamed_addr #2

attributes #0 = { uwtable "frame-pointer"="all" "probe-stack"="__rust_probestack" "target-cpu"="core2" }
attributes #1 = { nocallback nofree nosync nounwind readnone willreturn }
attributes #2 = { cold noinline noreturn uwtable "frame-pointer"="all" "probe-stack"="__rust_probestack" "target-cpu"="core2" }
attributes #3 = { noreturn }

!llvm.module.flags = !{!0}

!0 = !{i32 7, !"PIC Level", i32 2}
