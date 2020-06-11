	.text
	.file	"asm.7rcbfp3g-cgu.0"
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
.Lfunc_end0:
	.size	_ZN3std2rt10lang_start17hc6e896ad38043fd7E, .Lfunc_end0-_ZN3std2rt10lang_start17hc6e896ad38043fd7E
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
.Lfunc_end1:
	.size	_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17hadb40b3522af4983E, .Lfunc_end1-_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17hadb40b3522af4983E
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
.Lfunc_end2:
	.size	_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h64d90bcd22fa2e74E, .Lfunc_end2-_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h64d90bcd22fa2e74E
	.cfi_endproc

	.section	.text._ZN4core3ptr13drop_in_place17h33430eb2dafd4dd0E,"ax",@progbits
	.p2align	4, 0x90
	.type	_ZN4core3ptr13drop_in_place17h33430eb2dafd4dd0E,@function
_ZN4core3ptr13drop_in_place17h33430eb2dafd4dd0E:
	.cfi_startproc
	retq
.Lfunc_end3:
	.size	_ZN4core3ptr13drop_in_place17h33430eb2dafd4dd0E, .Lfunc_end3-_ZN4core3ptr13drop_in_place17h33430eb2dafd4dd0E
	.cfi_endproc

	.section	.text._ZN3asm11render_bool17hd4b005444cfe379bE,"ax",@progbits
	.p2align	4, 0x90
	.type	_ZN3asm11render_bool17hd4b005444cfe379bE,@function
_ZN3asm11render_bool17hd4b005444cfe379bE:
	.cfi_startproc
	testb	%dil, %dil
	je	.LBB4_2
	movl	$1702195828, (%rsi)
	retq
.LBB4_2:
	movl	$1936482662, (%rsi)
	movb	$101, 4(%rsi)
	retq
.Lfunc_end4:
	.size	_ZN3asm11render_bool17hd4b005444cfe379bE, .Lfunc_end4-_ZN3asm11render_bool17hd4b005444cfe379bE
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
	movl	$1, %edi
	callq	_ZN3asm11render_bool17hd4b005444cfe379bE
	movq	$0, (%rsp)
	movq	%rsp, %rsi
	xorl	%edi, %edi
	callq	_ZN3asm11render_bool17hd4b005444cfe379bE
	movq	$0, (%rsp)
	movq	%rsp, %rsi
	movl	$1, %edi
	callq	_ZN3asm11render_bool17hd4b005444cfe379bE
	popq	%rax
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end5:
	.size	_ZN3asm4main17hcf1a86dc0a4c595fE, .Lfunc_end5-_ZN3asm4main17hcf1a86dc0a4c595fE
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
.Lfunc_end6:
	.size	main, .Lfunc_end6-main
	.cfi_endproc

	.type	.L__unnamed_1,@object
	.section	.data.rel.ro..L__unnamed_1,"aw",@progbits
	.p2align	3
.L__unnamed_1:
	.quad	_ZN4core3ptr13drop_in_place17h33430eb2dafd4dd0E
	.quad	8
	.quad	8
	.quad	_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17hadb40b3522af4983E
	.quad	_ZN3std2rt10lang_start28_$u7b$$u7b$closure$u7d$$u7d$17hadb40b3522af4983E
	.quad	_ZN4core3ops8function6FnOnce40call_once$u7b$$u7b$vtable.shim$u7d$$u7d$17h64d90bcd22fa2e74E
	.size	.L__unnamed_1, 48


	.section	".note.GNU-stack","",@progbits
