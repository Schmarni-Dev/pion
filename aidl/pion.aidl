// IPion.aidl
package org.stardustxr.pion;

interface IPion {
    /**
     * Registers a file descriptor with its corresponding Binder reference.
     *
     * @param fd File descriptor to register (automatically closed after registration)
     * @param binderRef The Binder object reference to associate with this FD
     * @return true if registration succeeded
     */
    void register(in ParcelFileDescriptor fd, in IBinder binderRef);

    /**
     * Exchanges a file descriptor for its associated Binder reference.
     *
     * @param fd File descriptor to exchange (automatically closed after exchange)
     * @return BinderRefParcelable containing the associated binder, or null if not found
     */
    IBinder exchange(in ParcelFileDescriptor fd);
}
