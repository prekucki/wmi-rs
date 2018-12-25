use crate::connection::WMIConnection;
use failure::Error;
use log::debug;
use widestring::WideCString;
use winapi::shared::ntdef::NULL;
use std::ptr;
use std::ptr::Unique;
use winapi::um::wbemcli::{IWbemLocator, IWbemServices, IWbemClassObject, CLSID_WbemLocator, IID_IWbemLocator, IEnumWbemClassObject};
use winapi::shared::rpcdce::RPC_C_AUTHN_WINNT;
use winapi::shared::rpcdce::RPC_C_AUTHZ_NONE;
use winapi::shared::rpcdce::RPC_C_AUTHN_LEVEL_CALL;
use winapi::um::wbemcli::{WBEM_FLAG_FORWARD_ONLY, WBEM_FLAG_RETURN_IMMEDIATELY, WBEM_INFINITE};
use crate::utils::check_hres;
use widestring::WideCStr;
use winapi::um::oaidl::{VARIANT, VARIANT_n3};
use winapi::shared::wtypes::BSTR;
use std::mem;
use winapi::um::oleauto::VariantClear;


pub struct QueryResultEnumerator<'a> {
    wmi_con: &'a WMIConnection,
    p_enumerator: Option<Unique<IEnumWbemClassObject>>,

}

impl WMIConnection {
    pub fn query(&self, query: impl AsRef<str>) -> Result<QueryResultEnumerator, Error> {
        let query_language = WideCString::from_str("WQL")?;
        let query = WideCString::from_str(query)?;

        let mut p_enumerator = NULL as *mut IEnumWbemClassObject;

        unsafe {
            check_hres(
                (*self.svc()).ExecQuery(
                    query_language.as_ptr() as *mut _,
                    query.as_ptr() as *mut _,
                    (WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY) as i32,
                    ptr::null_mut(),
                    &mut p_enumerator)
            )?;
        }

        debug!("Got enumerator {:?}", p_enumerator);

        Ok(QueryResultEnumerator {
            wmi_con: self,
            p_enumerator: Unique::new(p_enumerator),
        })
    }
}

impl<'a> QueryResultEnumerator<'a> {
    pub fn p(&self) -> *mut IEnumWbemClassObject {
        self.p_enumerator.unwrap().as_ptr()
    }
}

impl<'a> Drop for QueryResultEnumerator<'a> {
    fn drop(&mut self) {
        if let Some(p_enumerator) = self.p_enumerator {
            unsafe {
                (*p_enumerator.as_ptr()).Release();
            }
        }
    }
}

impl<'a> Iterator for QueryResultEnumerator<'a> {
    type Item = Result<String, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut pcls_obj = NULL as *mut IWbemClassObject;
        let mut return_value = 0;

        if self.p_enumerator.is_none() {
            return None;
        }

        let res = unsafe {
            check_hres(
                (*self.p_enumerator.unwrap().as_ptr()).Next(WBEM_INFINITE as i32, 1,
                                     &mut pcls_obj,
                                     &mut return_value)
            )
        };

        if let Err(e) = res {
            return Some(Err(e.into()));
        }

        if return_value == 0 {
            return None;
        }

        debug!("Got enumerator {:?} and obj {:?}", self.p_enumerator, pcls_obj);

        let name_prop = WideCString::from_str("Caption").unwrap();
        let mut vt_prop: VARIANT = unsafe { mem::zeroed() };

        unsafe {
            (*pcls_obj).Get(
                name_prop.as_ptr() as *mut _,
                0,
                &mut vt_prop,
                ptr::null_mut(),
                ptr::null_mut(),
            );
        }

        let p = unsafe { vt_prop.n1.n2().n3.bstrVal() };

        let prop_val: &WideCStr = unsafe {
            WideCStr::from_ptr_str(*p)
        };

        unsafe { VariantClear(&mut vt_prop) };

        // TODO: Remove this unwrap.
        let property_value_as_string = prop_val.to_string().unwrap();

        debug!("Got {}", property_value_as_string);

        unsafe {
            (*pcls_obj).Release();
        }

        Some(Ok(property_value_as_string))
    }
}