#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/init.h>

static int __init linuxab_init(void)
{
    printk(KERN_INFO "linuxab: loaded\n");
    return 0;
}

static void __exit linuxab_exit(void)
{
    printk(KERN_INFO "linuxab: unloaded\n");
}

module_init(linuxab_init);
module_exit(linuxab_exit);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Jatin");
MODULE_DESCRIPTION("linuxab kernel module");
