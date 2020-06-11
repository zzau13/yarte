Use unaligned `movl` for render booleans
```
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
```