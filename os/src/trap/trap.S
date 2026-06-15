.altmacro
    .macro SAVE_GP n
        sd x\n, \n*8(sp)
    .endm
    .macro RESTORE_GP n
        ld x\n, \n*8(sp)
    .endm

    # 注意：这里切换到了专门的 .text.trampoline 段
    .section .text.trampoline
    .globl __alltraps
    .globl __restore
    .globl strampoline
    .align 2
strampoline:
__alltraps:
    # 1. 交换 sscratch 和 sp，现在 sp 指向用户空间最高的 TRAP_CONTEXT 虚拟地址
    csrrw sp, sscratch, sp
    
    # 2. 在 TrapContext 中保存所有用户通用寄存器
    sd x1, 1*8(sp)
    # 跳过 sp(x2)
    sd x3, 3*8(sp)
    .set n, 4
    .rept 28
        SAVE_GP %n
        .set n, n+1
    .endr
    
    # 3. 保存控制和状态寄存器（sstatus 和 sepc）
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    
    # 4. 从 TrapContext 中读取内核先前埋好的控制信息
    ld t0, 34*8(sp) # kernel_satp (内核页表的根节点物理页号)
    ld t1, 35*8(sp) # trap_handler (内核中断处理函数的入口)
    ld sp, 36*8(sp) # kernel_sp (内核栈的虚拟地址)
    
    # 5. 切换到内核页表并刷新快表 (TLB)
    csrw satp, t0
    sfence.vma
    
    # 6. 跳转到内核的绝对物理/虚拟地址去执行具体的 trap_handler
    jr t1

__restore:
    # 此时 a0 传入的是应用 TrapContext 在其虚拟空间中的虚拟地址
    # a1 传入的是该应用对应的 satp 页表令牌
    
    # 1. 切换回应用的用户页表并刷新快表
    csrw satp, a1
    sfence.vma
    
    # 2. 让 sscratch 重新盯住该应用的 TrapContext 虚拟地址
    csrw sscratch, a0
    mv sp, a0
    
    # 3. 从 TrapContext 恢复寄存器状态
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 4
    .rept 28
        RESTORE_GP %n
        .set n, n+1
    .endr
    
    # 4. 恢复用户自己的栈指针 sp
    ld sp, 2*8(sp)
    sret
