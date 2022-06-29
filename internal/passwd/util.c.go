package passwd

/*
#include <nss.h>
#include <pwd.h>
*/
import "C"
import (
	"bytes"
	"unsafe"

	"github.com/ubuntu/aad-auth/internal/nss"
)

type (
	// CPasswd is the struct passwd
	CPasswd = *C.struct_passwd
	// CChat allow to cast to a char
	CChar = C.char
	// CSizeT allow to cast to a size_t
	CSizeT = C.size_t
)

// ToCpasswd transforms the Go passwd struct to a C struct passwd, filling buffer, result and nss_status.
// The function will check first for errors to transform them to corresponding nss status.
func (p Passwd) ToCpasswd(pwd CPasswd, buf *CChar, buflen CSizeT, result *CPasswd) error {
	// result points to NULL in case of error
	*result = (*C.struct_passwd)(nil)

	// Ensure the buffer is big enough for all fields of passwd, with an offset.
	// 5 is the number of fields of type char * in the structure 'passwd'
	if int(buflen) < len(p.name)+len(p.passwd)+len(p.gecos)+len(p.dir)+len(p.shell)+5 {
		return nss.ErrTryAgain
	}

	// Transform the C guffer to a Go one.
	gobuf := C.GoBytes(unsafe.Pointer(buf), C.int(buflen))
	b := bytes.NewBuffer(gobuf)
	b.Reset()

	// Points the C passwd struct field to the current address of the buffer (start of current field value),
	// then file the buffer with the value we want to use.
	pwd.pw_name = (*C.char)(unsafe.Pointer(&gobuf[b.Len()]))
	b.WriteString(p.name)
	b.WriteByte(0)

	pwd.pw_passwd = (*C.char)(unsafe.Pointer(&gobuf[b.Len()]))
	b.WriteString(p.passwd)
	b.WriteByte(0)

	pwd.pw_gecos = (*C.char)(unsafe.Pointer(&gobuf[b.Len()]))
	b.WriteString(p.gecos)
	b.WriteByte(0)

	pwd.pw_dir = (*C.char)(unsafe.Pointer(&gobuf[b.Len()]))
	b.WriteString(p.dir)
	b.WriteByte(0)

	pwd.pw_shell = (*C.char)(unsafe.Pointer(&gobuf[b.Len()]))
	b.WriteString(p.shell)
	b.WriteByte(0)

	// uid and git are not pointers, but just the uint itself.
	pwd.pw_uid = C.uint(p.uid)
	pwd.pw_gid = C.uint(p.gid)

	// Point our result pointer struct to our C passwd.
	*result = (*C.struct_passwd)(unsafe.Pointer(&pwd))

	return nil
}
