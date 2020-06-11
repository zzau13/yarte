	.text
	.file	"asm.7rcbfp3g-cgu.0"
	.section	".text._ZN36_$LT$T$u20$as$u20$core..any..Any$GT$7type_id17hac62011ec9ba2153E","ax",@progbits
	.p2align	4, 0x90
	.type	_ZN36_$LT$T$u20$as$u20$core..any..Any$GT$7type_id17hac62011ec9ba2153E,@function
_ZN36_$LT$T$u20$as$u20$core..any..Any$GT$7type_id17hac62011ec9ba2153E:
	.cfi_startproc
	movabsq	$1229646359891580772, %rax
	retq
.Lfunc_end0:
	.size	_ZN36_$LT$T$u20$as$u20$core..any..Any$GT$7type_id17hac62011ec9ba2153E, .Lfunc_end0-_ZN36_$LT$T$u20$as$u20$core..any..Any$GT$7type_id17hac62011ec9ba2153E
	.cfi_endproc

	.section	.text._ZN3std2rt10lang_start17hc6e896ad38043fd7E,"ax",@progbits
	.hidden	_ZN3std2rt10lang_start17hc6e896ad38043fd7E
	.globl	_ZN3std2rt10lang_start17hc6e896ad38043fd7E
	.p2align	4, 0x90
	.type	_ZN3std2rt10lang_start17hc6e896ad38043fd7E,@function
_ZN3std2rt10lang_start17hc6e896ad38043fd7E:
	.cfi_startproc
	pushq	%rax
	.cfi_def_cfa_offset 16
	movq	%rdx, %rcx
	movq	%rsi, %rdx
	movq	%rdi, (%rsp)
	leaq	.L__unnamed_1(%rip), %rsi
	movq	%rsp, %rdi
	callq	*_ZN3std2rt19lang_start_internal17hcf7fb98a775d5af0E@GOTPCREL(%rip)
	popq	%rcx
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end1:
	.size	_ZN3std2rt10lang_start17hc6e896ad38043fd7E, .Lfunc_end1-_ZN3std2rt10lang_start17hc6e896ad38043fd7E
	.cfi_endproc

	.section	".text._ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17hadb40b3522af4983E","ax",@progbits
	.p2align	4, 0x90
	.type	_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17hadb40b3522af4983E,@function
_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17hadb40b3522af4983E:
	.cfi_startproc
	pushq	%rax
	.cfi_def_cfa_offset 16
	callq	*(%rdi)
	xorl	%eax, %eax
	popq	%rcx
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end2:
	.size	_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17hadb40b3522af4983E, .Lfunc_end2-_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17hadb40b3522af4983E
	.cfi_endproc

	.section	.text._ZN3std9panicking11begin_panic17h86798402bcbec000E,"ax",@progbits
	.p2align	4, 0x90
	.type	_ZN3std9panicking11begin_panic17h86798402bcbec000E,@function
_ZN3std9panicking11begin_panic17h86798402bcbec000E:
	.cfi_startproc
	subq	$24, %rsp
	.cfi_def_cfa_offset 32
	leaq	.L__unnamed_2(%rip), %rax
	movq	%rax, 8(%rsp)
	movq	$14, 16(%rsp)
	leaq	.L__unnamed_3(%rip), %rdi
	callq	*_ZN4core5panic8Location6caller17h2552a4c1bcb3d368E@GOTPCREL(%rip)
	leaq	.L__unnamed_4(%rip), %rsi
	leaq	8(%rsp), %rdi
	xorl	%edx, %edx
	movq	%rax, %rcx
	callq	*_ZN3std9panicking20rust_panic_with_hook17hc0b4730bb8013f9dE@GOTPCREL(%rip)
	ud2
.Lfunc_end3:
	.size	_ZN3std9panicking11begin_panic17h86798402bcbec000E, .Lfunc_end3-_ZN3std9panicking11begin_panic17h86798402bcbec000E
	.cfi_endproc

	.section	".text._ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h64d90bcd22fa2e74E","ax",@progbits
	.p2align	4, 0x90
	.type	_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h64d90bcd22fa2e74E,@function
_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h64d90bcd22fa2e74E:
	.cfi_startproc
	pushq	%rax
	.cfi_def_cfa_offset 16
	callq	*(%rdi)
	xorl	%eax, %eax
	popq	%rcx
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end4:
	.size	_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h64d90bcd22fa2e74E, .Lfunc_end4-_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h64d90bcd22fa2e74E
	.cfi_endproc

	.section	.text._ZN4core3ptr13drop_in_place17h317e32f0f1ae3c2dE,"ax",@progbits
	.p2align	4, 0x90
	.type	_ZN4core3ptr13drop_in_place17h317e32f0f1ae3c2dE,@function
_ZN4core3ptr13drop_in_place17h317e32f0f1ae3c2dE:
	.cfi_startproc
	retq
.Lfunc_end5:
	.size	_ZN4core3ptr13drop_in_place17h317e32f0f1ae3c2dE, .Lfunc_end5-_ZN4core3ptr13drop_in_place17h317e32f0f1ae3c2dE
	.cfi_endproc

	.section	".text._ZN91_$LT$std..panicking..begin_panic..PanicPayload$LT$A$GT$$u20$as$u20$core..panic..BoxMeUp$GT$3get17h9414b84277c4ac72E","ax",@progbits
	.p2align	4, 0x90
	.type	_ZN91_$LT$std..panicking..begin_panic..PanicPayload$LT$A$GT$$u20$as$u20$core..panic..BoxMeUp$GT$3get17h9414b84277c4ac72E,@function
_ZN91_$LT$std..panicking..begin_panic..PanicPayload$LT$A$GT$$u20$as$u20$core..panic..BoxMeUp$GT$3get17h9414b84277c4ac72E:
	.cfi_startproc
	pushq	%rax
	.cfi_def_cfa_offset 16
	cmpq	$0, (%rdi)
	je	.LBB6_1
	movq	%rdi, %rax
	leaq	.L__unnamed_5(%rip), %rdx
	popq	%rcx
	.cfi_def_cfa_offset 8
	retq
.LBB6_1:
	.cfi_def_cfa_offset 16
	callq	*_ZN3std7process5abort17ha0f487946a23ecfcE@GOTPCREL(%rip)
	ud2
.Lfunc_end6:
	.size	_ZN91_$LT$std..panicking..begin_panic..PanicPayload$LT$A$GT$$u20$as$u20$core..panic..BoxMeUp$GT$3get17h9414b84277c4ac72E, .Lfunc_end6-_ZN91_$LT$std..panicking..begin_panic..PanicPayload$LT$A$GT$$u20$as$u20$core..panic..BoxMeUp$GT$3get17h9414b84277c4ac72E
	.cfi_endproc

	.section	".text._ZN91_$LT$std..panicking..begin_panic..PanicPayload$LT$A$GT$$u20$as$u20$core..panic..BoxMeUp$GT$8take_box17hcc2d6da81d436228E","ax",@progbits
	.p2align	4, 0x90
	.type	_ZN91_$LT$std..panicking..begin_panic..PanicPayload$LT$A$GT$$u20$as$u20$core..panic..BoxMeUp$GT$8take_box17hcc2d6da81d436228E,@function
_ZN91_$LT$std..panicking..begin_panic..PanicPayload$LT$A$GT$$u20$as$u20$core..panic..BoxMeUp$GT$8take_box17hcc2d6da81d436228E:
	.cfi_startproc
	pushq	%r14
	.cfi_def_cfa_offset 16
	pushq	%rbx
	.cfi_def_cfa_offset 24
	pushq	%rax
	.cfi_def_cfa_offset 32
	.cfi_offset %rbx, -24
	.cfi_offset %r14, -16
	movq	(%rdi), %rbx
	movq	8(%rdi), %r14
	movq	$0, (%rdi)
	testq	%rbx, %rbx
	je	.LBB7_3
	movl	$16, %edi
	movl	$8, %esi
	callq	*__rust_alloc@GOTPCREL(%rip)
	testq	%rax, %rax
	je	.LBB7_4
	movq	%rbx, (%rax)
	movq	%r14, 8(%rax)
	leaq	.L__unnamed_5(%rip), %rdx
	addq	$8, %rsp
	.cfi_def_cfa_offset 24
	popq	%rbx
	.cfi_def_cfa_offset 16
	popq	%r14
	.cfi_def_cfa_offset 8
	retq
.LBB7_3:
	.cfi_def_cfa_offset 32
	callq	*_ZN3std7process5abort17ha0f487946a23ecfcE@GOTPCREL(%rip)
	ud2
.LBB7_4:
	movl	$16, %edi
	movl	$8, %esi
	callq	*_ZN5alloc5alloc18handle_alloc_error17h4708e9620f23812bE@GOTPCREL(%rip)
	ud2
.Lfunc_end7:
	.size	_ZN91_$LT$std..panicking..begin_panic..PanicPayload$LT$A$GT$$u20$as$u20$core..panic..BoxMeUp$GT$8take_box17hcc2d6da81d436228E, .Lfunc_end7-_ZN91_$LT$std..panicking..begin_panic..PanicPayload$LT$A$GT$$u20$as$u20$core..panic..BoxMeUp$GT$8take_box17hcc2d6da81d436228E
	.cfi_endproc

	.section	.text._ZN3asm11render_bool17hd4b005444cfe379bE,"ax",@progbits
	.p2align	4, 0x90
	.type	_ZN3asm11render_bool17hd4b005444cfe379bE,@function
_ZN3asm11render_bool17hd4b005444cfe379bE:
	.cfi_startproc
	testb	%dil, %dil
	je	.LBB8_4
	cmpq	$4, %rdx
	jb	.LBB8_2
	movl	$1702195828, (%rsi)
	movl	$1, %eax
	movl	$4, %edx
	retq
.LBB8_4:
	cmpq	$5, %rdx
	jae	.LBB8_5
.LBB8_2:
	xorl	%eax, %eax
	retq
.LBB8_5:
	movl	$1936482662, (%rsi)
	movb	$101, 4(%rsi)
	movl	$1, %eax
	movl	$5, %edx
	retq
.Lfunc_end8:
	.size	_ZN3asm11render_bool17hd4b005444cfe379bE, .Lfunc_end8-_ZN3asm11render_bool17hd4b005444cfe379bE
	.cfi_endproc

	.section	.text._ZN3asm4main17hcf1a86dc0a4c595fE,"ax",@progbits
	.p2align	4, 0x90
	.type	_ZN3asm4main17hcf1a86dc0a4c595fE,@function
_ZN3asm4main17hcf1a86dc0a4c595fE:
	.cfi_startproc
	pushq	%rax
	.cfi_def_cfa_offset 16
	movq	$0, (%rsp)
	movq	%rsp, %rsi
	movl	$8, %edx
	movl	$1, %edi
	callq	_ZN3asm11render_bool17hd4b005444cfe379bE
	testq	%rax, %rax
	je	.LBB9_1
	movq	$0, (%rsp)
	movq	%rsp, %rsi
	movl	$8, %edx
	xorl	%edi, %edi
	callq	_ZN3asm11render_bool17hd4b005444cfe379bE
	testq	%rax, %rax
	je	.LBB9_4
	leaq	.L__unnamed_6(%rip), %rsi
	movl	$1, %edi
	xorl	%edx, %edx
	callq	_ZN3asm11render_bool17hd4b005444cfe379bE
	cmpq	$1, %rax
	je	.LBB9_7
	popq	%rax
	.cfi_def_cfa_offset 8
	retq
.LBB9_1:
	.cfi_def_cfa_offset 16
	leaq	.L__unnamed_7(%rip), %rdi
	leaq	.L__unnamed_8(%rip), %rdx
	jmp	.LBB9_2
.LBB9_4:
	leaq	.L__unnamed_7(%rip), %rdi
	leaq	.L__unnamed_9(%rip), %rdx
.LBB9_2:
	movl	$43, %esi
	callq	*_ZN4core9panicking5panic17h57854ec308e1f8abE@GOTPCREL(%rip)
	ud2
.LBB9_7:
	callq	_ZN3std9panicking11begin_panic17h86798402bcbec000E
	ud2
.Lfunc_end9:
	.size	_ZN3asm4main17hcf1a86dc0a4c595fE, .Lfunc_end9-_ZN3asm4main17hcf1a86dc0a4c595fE
	.cfi_endproc

	.section	.text.main,"ax",@progbits
	.globl	main
	.p2align	4, 0x90
	.type	main,@function
main:
	.cfi_startproc
	pushq	%rax
	.cfi_def_cfa_offset 16
	movq	%rsi, %rcx
	movslq	%edi, %rdx
	leaq	_ZN3asm4main17hcf1a86dc0a4c595fE(%rip), %rax
	movq	%rax, (%rsp)
	leaq	.L__unnamed_1(%rip), %rsi
	movq	%rsp, %rdi
	callq	*_ZN3std2rt19lang_start_internal17hcf7fb98a775d5af0E@GOTPCREL(%rip)
	popq	%rcx
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end10:
	.size	main, .Lfunc_end10-main
	.cfi_endproc

	.type	.L__unnamed_1,@object
	.section	.data.rel.ro..L__unnamed_1,"aw",@progbits
	.p2align	3
.L__unnamed_1:
	.quad	_ZN4core3ptr13drop_in_place17h317e32f0f1ae3c2dE
	.quad	8
	.quad	8
	.quad	_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17hadb40b3522af4983E
	.quad	_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17hadb40b3522af4983E
	.quad	_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h64d90bcd22fa2e74E
	.size	.L__unnamed_1, 48

	.type	.L__unnamed_4,@object
	.section	.data.rel.ro..L__unnamed_4,"aw",@progbits
	.p2align	3
.L__unnamed_4:
	.quad	_ZN4core3ptr13drop_in_place17h317e32f0f1ae3c2dE
	.quad	16
	.quad	8
	.quad	_ZN91_$LT$std..panicking..begin_panic..PanicPayload$LT$A$GT$$u20$as$u20$core..panic..BoxMeUp$GT$8take_box17hcc2d6da81d436228E
	.quad	_ZN91_$LT$std..panicking..begin_panic..PanicPayload$LT$A$GT$$u20$as$u20$core..panic..BoxMeUp$GT$3get17h9414b84277c4ac72E
	.size	.L__unnamed_4, 40

	.type	.L__unnamed_7,@object
	.section	.rodata..L__unnamed_7,"a",@progbits
.L__unnamed_7:
	.ascii	"called `Option::unwrap()` on a `None` value"
	.size	.L__unnamed_7, 43

	.type	.L__unnamed_5,@object
	.section	.data.rel.ro..L__unnamed_5,"aw",@progbits
	.p2align	3
.L__unnamed_5:
	.quad	_ZN4core3ptr13drop_in_place17h317e32f0f1ae3c2dE
	.quad	16
	.quad	8
	.quad	_ZN36_$LT$T$u20$as$u20$core..any..Any$GT$7type_id17hac62011ec9ba2153E
	.size	.L__unnamed_5, 32

	.type	.L__unnamed_10,@object
	.section	.rodata..L__unnamed_10,"a",@progbits
.L__unnamed_10:
	.ascii	"asm.rs"
	.size	.L__unnamed_10, 6

	.type	.L__unnamed_8,@object
	.section	.data.rel.ro..L__unnamed_8,"aw",@progbits
	.p2align	3
.L__unnamed_8:
	.quad	.L__unnamed_10
	.asciz	"\006\000\000\000\000\000\000\000\036\000\000\000\t\000\000"
	.size	.L__unnamed_8, 24

	.type	.L__unnamed_9,@object
	.section	.data.rel.ro..L__unnamed_9,"aw",@progbits
	.p2align	3
.L__unnamed_9:
	.quad	.L__unnamed_10
	.asciz	"\006\000\000\000\000\000\000\000\037\000\000\000\t\000\000"
	.size	.L__unnamed_9, 24

	.type	.L__unnamed_6,@object
	.section	.rodata..L__unnamed_6,"a",@progbits
.L__unnamed_6:
	.size	.L__unnamed_6, 0

	.type	.L__unnamed_2,@object
	.section	.rodata..L__unnamed_2,"a",@progbits
.L__unnamed_2:
	.ascii	"explicit panic"
	.size	.L__unnamed_2, 14

	.type	.L__unnamed_3,@object
	.section	.data.rel.ro..L__unnamed_3,"aw",@progbits
	.p2align	3
.L__unnamed_3:
	.quad	.L__unnamed_10
	.asciz	"\006\000\000\000\000\000\000\000!\000\000\000\r\000\000"
	.size	.L__unnamed_3, 24


	.section	".note.GNU-stack","",@progbits
