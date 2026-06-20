#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/init.h>
#include <linux/fs.h>
#include <linux/device.h>
#include <linux/slab.h>
#include <linux/uaccess.h>
#include <linux/mm.h>
#include <immintrin.h>

#define DEVICE_NAME "linuxab"
#define CLASS_NAME "linuxab"
#define TENSOR_SIZE (16 * 1024 * 1024)

static int major;
static struct class *linuxab_class;
static struct device *linuxab_device;
static float *tensor_mem;

static void matmul_avx2(float *a, float *b, float *c, int n)
{
    int i, j, k;
    for (i = 0; i < n; i += 8) {
        for (j = 0; j < n; j++) {
            __m256 sum = _mm256_setzero_ps();
            for (k = 0; k < n; k += 8) {
                __m256 va = _mm256_loadu_ps(&a[i * n + k]);
                __m256 vb = _mm256_loadu_ps(&b[k * n + j]);
                sum = _mm256_fmadd_ps(va, vb, sum);
            }
            _mm256_storeu_ps(&c[i * n + j], sum);
        }
    }
}

static int linuxab_open(struct inode *inode, struct file *file)
{
    return 0;
}

static int linuxab_release(struct inode *inode, struct file *file)
{
    return 0;
}

static long linuxab_ioctl(struct file *file, unsigned int cmd, unsigned long arg)
{
    if (cmd == 1) {
        matmul_avx2(tensor_mem, tensor_mem + 4096, tensor_mem + 8192, 64);
        printk(KERN_INFO "linuxab: avx2 matmul done\n");
    }
    return 0;
}

static int linuxab_mmap(struct file *file, struct vm_area_struct *vma)
{
    unsigned long size = vma->vm_end - vma->vm_start;
    if (remap_pfn_range(vma, vma->vm_start,
                       virt_to_phys(tensor_mem) >> PAGE_SHIFT,
                       size, vma->vm_page_prot))
        return -EAGAIN;
    return 0;
}

static struct file_operations fops = {
    .owner = THIS_MODULE,
    .open = linuxab_open,
    .release = linuxab_release,
    .unlocked_ioctl = linuxab_ioctl,
    .mmap = linuxab_mmap,
};

static int __init linuxab_init(void)
{
    major = register_chrdev(0, DEVICE_NAME, &fops);
    linuxab_class = class_create(THIS_MODULE, CLASS_NAME);
    linuxab_device = device_create(linuxab_class, NULL, MKDEV(major, 0), NULL, DEVICE_NAME);
    tensor_mem = kzalloc(TENSOR_SIZE, GFP_KERNEL);
    printk(KERN_INFO "linuxab: avx2 accel loaded\n");
    return 0;
}

static void __exit linuxab_exit(void)
{
    kfree(tensor_mem);
    device_destroy(linuxab_class, MKDEV(major, 0));
    class_destroy(linuxab_class);
    unregister_chrdev(major, DEVICE_NAME);
    printk(KERN_INFO "linuxab: unloaded\n");
}

module_init(linuxab_init);
module_exit(linuxab_exit);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Jatin");
MODULE_DESCRIPTION("linuxab avx2 accel");
