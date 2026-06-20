#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/fs.h>
#include <linux/device.h>
#include <linux/slab.h>
#include <linux/uaccess.h>
#include <linux/mm.h>

#define DEVICE_NAME "linuxab_accel"
#define CLASS_NAME "linuxab_accel"
#define TENSOR_MAX_SIZE (16 * 1024 * 1024) // 16MB max tensor

struct accel_cmd {
    __u32 op;       // 0=matmul, 1=conv, 2=relu, 3=softmax
    __u32 dtype;    // 0=f32, 1=f16, 2=i8
    __u64 in_size;
    __u64 out_size;
    __u64 w_size;
};

static int major;
static struct class *accel_class;
static struct device *accel_dev;
static void *tensor_buf;

static long linuxab_accel_ioctl(struct file *file, unsigned int cmd, unsigned long arg)
{
    struct accel_cmd ac;
    void __user *user_in, *user_out, *user_w;

    if (copy_from_user(&ac, (void __user *)arg, sizeof(ac)))
        return -EFAULT;

    if (ac.in_size > TENSOR_MAX_SIZE || ac.out_size > TENSOR_MAX_SIZE || ac.w_size > TENSOR_MAX_SIZE)
        return -EINVAL;

    user_in = (void __user *)(arg + sizeof(ac));
    user_out = (void __user *)(arg + sizeof(ac) + ac.in_size);
    user_w = (void __user *)(arg + sizeof(ac) + ac.in_size + ac.out_size);

    switch (ac.op) {
    case 0: // matmul
        printk(KERN_INFO "linuxab_accel: matmul %lux%lu f%d\n",
               ac.in_size, ac.out_size, ac.dtype == 0 ? 32 : (ac.dtype == 1 ? 16 : 8));
        // SIMD-optimized matmul goes here
        break;
    case 1: // conv
        printk(KERN_INFO "linuxab_accel: conv2d\n");
        break;
    case 2: // relu
        printk(KERN_INFO "linuxab_accel: relu\n");
        break;
    case 3: // softmax
        printk(KERN_INFO "linuxab_accel: softmax\n");
        break;
    default:
        return -EINVAL;
    }

    return 0;
}

static int linuxab_accel_mmap(struct file *file, struct vm_area_struct *vma)
{
    unsigned long size = vma->vm_end - vma->vm_start;

    if (size > TENSOR_MAX_SIZE)
        return -EINVAL;

    return remap_pfn_range(vma, vma->vm_start,
                          virt_to_phys(tensor_buf) >> PAGE_SHIFT,
                          size, vma->vm_page_prot);
}

static int linuxab_accel_open(struct inode *inode, struct file *file)
{
    return 0;
}

static int linuxab_accel_release(struct inode *inode, struct file *file)
{
    return 0;
}

static struct file_operations fops = {
    .owner = THIS_MODULE,
    .open = linuxab_accel_open,
    .release = linuxab_accel_release,
    .unlocked_ioctl = linuxab_accel_ioctl,
    .mmap = linuxab_accel_mmap,
};

static int __init linuxab_accel_init(void)
{
    major = register_chrdev(0, DEVICE_NAME, &fops);
    if (major < 0)
        return major;

    accel_class = class_create(THIS_MODULE, CLASS_NAME);
    if (IS_ERR(accel_class)) {
        unregister_chrdev(major, DEVICE_NAME);
        return PTR_ERR(accel_class);
    }

    accel_dev = device_create(accel_class, NULL, MKDEV(major, 0), NULL, DEVICE_NAME);
    if (IS_ERR(accel_dev)) {
        class_destroy(accel_class);
        unregister_chrdev(major, DEVICE_NAME);
        return PTR_ERR(accel_dev);
    }

    tensor_buf = kzalloc(TENSOR_MAX_SIZE, GFP_KERNEL);
    if (!tensor_buf)
        return -ENOMEM;

    printk(KERN_INFO "linuxab_accel: loaded at /dev/linuxab_accel\n");
    return 0;
}

static void __exit linuxab_accel_exit(void)
{
    kfree(tensor_buf);
    device_destroy(accel_class, MKDEV(major, 0));
    class_destroy(accel_class);
    unregister_chrdev(major, DEVICE_NAME);
    printk(KERN_INFO "linuxab_accel: unloaded\n");
}

module_init(linuxab_accel_init);
module_exit(linuxab_accel_exit);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Jatin / linuxab");
MODULE_DESCRIPTION("linuxab AI acceleration kernel module");
