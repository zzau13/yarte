Use unaligned `movl` for render booleans
```
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
```
