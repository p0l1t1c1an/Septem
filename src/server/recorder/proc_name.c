
#include <sys/types.h>
#include <sys/user.h>
#include <stdlib.h>
#include <string.h>

#ifdef __FreeBSD__
#include <libutil.h>

#elif __OpenBSD__
#include <sys/param.h>
#include <sys/sysctl.h>

// Semi-clone of Freebsd's kinfo_getproc in libutil.h 
static struct kinfo_proc *    
kinfo_getproc(pid_t pid)    
{ 
    struct kinfo_proc *kproc;    
    size_t len = 0; 
    int mib[6] = {CTL_KERN, KERN_PROC, KERN_PROC_PID, pid, (int) sizeof(struct kinfo_proc), 1};    
    
    if (sysctl(mib, nitems(mib), NULL, &len, NULL, 0) < 0)    
        return NULL;    
   
    kproc = malloc(len); 
    if (kproc == NULL) 
        return NULL; 
    
    if (sysctl(mib, nitems(mib), kproc, &len, NULL, 0) < 0) 
        goto bad; 
    if (len != sizeof(*kproc))
		goto bad;
    if (kproc->p_pid != pid)    
        goto bad;
    return kproc; 
    
bad:    
    free(kproc);    
    return NULL;    
}
#endif

#if defined (__FreeBSD__) || defined (__OpenBSD__)

// Why can't they follow same naming convention
static char * 
get_proc_name(struct kinfo_proc *p) {
#ifdef __FreeBSD__
    return p->ki_comm;
#elif __OpenBSD__
    return p->p_comm;
#endif
}

int 
proc_name(char **kproc_name, pid_t pid)
{
	int ret_val = 2;
	struct kinfo_proc *kproc = kinfo_getproc(pid);
	if(kproc) {
		ret_val--;
		size_t len = strlen(get_proc_name(kproc));
		if(len) {
		    *kproc_name = malloc(len+1);
			strcpy(*kproc_name, get_proc_name(kproc));
	    	ret_val--;
		}
		free(kproc);
	}
	return ret_val;
}

#endif

