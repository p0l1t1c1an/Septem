
#include <sys/types.h>
#include <sys/user.h>
#include <libutil.h>
#include <stdlib.h>
#include <string.h>

int proc_name(char **kproc_name, pid_t pid)
{
	int ret_val = 2;
	struct kinfo_proc *kproc = kinfo_getproc(pid);
	if(kproc) {
		ret_val--;
		size_t len = strlen(kproc->ki_comm);
		if(len) {
			*kproc_name = malloc(len+1);
			strcpy(*kproc_name, kproc->ki_comm);
	    	ret_val--;
		}
		free(kproc);
	}
	return ret_val;
}

