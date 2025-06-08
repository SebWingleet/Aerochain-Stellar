#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, contracterror,
    Address, Env, String, Map, Symbol, Vec, 
    symbol_short, log
};

// Définition des symboles pour les clés de stockage
const ADMINS: Symbol = symbol_short!("ADMINS");
const OEM_ORGS: Symbol = symbol_short!("OEM_ORGS");
const MRO_ORGS: Symbol = symbol_short!("MRO_ORGS");
const PARTS: Symbol = symbol_short!("PARTS");

// Types d'organisations autorisées
#[derive(Clone, Copy)]
#[contracttype]
pub enum OrgType {
    OEM,     // Original Equipment Manufacturer
    MRO,     // Maintenance, Repair, and Operations
    Airline, // Compagnie aérienne
    Lessor,  // Société de leasing
    Distributor, // Distributeur certifié
}

// Structure d'une organisation
#[derive(Clone)]
#[contracttype]
pub struct Organization {
    pub id: Address,
    pub name: String,
    pub org_type: OrgType,
    pub certificates: Vec<String>,
    pub active: bool,
}

// Statut d'une pièce
#[derive(Clone, Copy, PartialEq)]
#[contracttype]
pub enum PartStatus {
    Active,
    InMaintenance,
    Retired,
    Quarantined,
}

// Structure d'une pièce aéronautique
#[contracttype]
#[derive(Clone)]
pub struct AeronauticPart {
    pub uid: String,
    pub part_number: String,
    pub serial_number: String,
    pub manufacturer: Address,
    pub date_of_manufacture: u64, // Timestamp Unix
    pub current_owner: Address,
    pub status: PartStatus,
    pub total_hours: u32,
    pub total_cycles: u32,
    pub last_updated: u64, // Timestamp Unix
    pub document_hashes: Map<String, String>, // Nom du document -> Hash
}

// Erreurs possibles - utilisation de contracterror
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    OrgNotRegistered = 2,
    NotAnOEM = 3,
    PartAlreadyExists = 4,
    PartNotFound = 5,
    InvalidInput = 6,
}

#[contract]
pub struct PartsRegistry;

#[contractimpl]
impl PartsRegistry {
    // Fonction d'initialisation du contrat
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        // Vérifier que le contrat n'est pas déjà initialisé
        if env.storage().instance().has(&ADMINS) {
            return Err(Error::InvalidInput);
        }
        
        // Vérifier l'identité de l'administrateur
        admin.require_auth();
        
        // Enregistrer l'administrateur
        let mut admins = Vec::new(&env);
        admins.push_back(admin.clone());
        env.storage().instance().set(&ADMINS, &admins);
        
        // Initialiser les collections
        let oem_orgs: Vec<Organization> = Vec::new(&env);
        let mro_orgs: Vec<Organization> = Vec::new(&env);
        let parts: Map<String, AeronauticPart> = Map::new(&env);
        
        env.storage().instance().set(&OEM_ORGS, &oem_orgs);
        env.storage().instance().set(&MRO_ORGS, &mro_orgs);
        env.storage().instance().set(&PARTS, &parts);
        
        // Étendre la durée de vie du stockage (5 ans en nombre de ledgers, estimation)
        // Avec un ledger toutes les 5 secondes: 5 ans ≈ 31,536,000 secondes / 5 = 6,307,200 ledgers
        env.storage().instance().extend_ttl(1000, 6_307_200);
        
        log!(&env, "Contract initialized with admin: {}", admin);
        Ok(())
    }
    
    // Enregistrer une nouvelle organisation OEM
    pub fn register_oem(
        env: Env, 
        caller: Address, 
        org_address: Address, 
        name: String, 
        certificates: Vec<String>
    ) -> Result<(), Error> {
        // Vérifier que l'appelant est un administrateur
        caller.require_auth();
        Self::ensure_is_admin(&env, &caller)?;
        
        // Créer l'organisation
        let org = Organization {
            id: org_address.clone(),
            name,
            org_type: OrgType::OEM,
            certificates,
            active: true,
        };
        
        // Récupérer et mettre à jour la liste des OEMs
        let mut oem_orgs: Vec<Organization> = env.storage().instance().get(&OEM_ORGS).unwrap_or(Vec::new(&env));
        oem_orgs.push_back(org);
        env.storage().instance().set(&OEM_ORGS, &oem_orgs);
        
        log!(&env, "Registered new OEM: {}", org_address);
        Ok(())
    }
    
    // Enregistrer une nouvelle organisation MRO
    pub fn register_mro(
        env: Env, 
        caller: Address, 
        org_address: Address, 
        name: String, 
        certificates: Vec<String>
    ) -> Result<(), Error> {
        // Vérifier que l'appelant est un administrateur
        caller.require_auth();
        Self::ensure_is_admin(&env, &caller)?;
        
        // Créer l'organisation
        let org = Organization {
            id: org_address.clone(),
            name,
            org_type: OrgType::MRO,
            certificates,
            active: true,
        };
        
        // Récupérer et mettre à jour la liste des MROs
        let mut mro_orgs: Vec<Organization> = env.storage().instance().get(&MRO_ORGS).unwrap_or(Vec::new(&env));
        mro_orgs.push_back(org);
        env.storage().instance().set(&MRO_ORGS, &mro_orgs);
        
        log!(&env, "Registered new MRO: {}", org_address);
        Ok(())
    }
    
    // Créer une nouvelle pièce aéronautique
    pub fn create_part(
        env: Env,
        manufacturer: Address,
        uid: String,
        part_number: String,
        serial_number: String,
        document_hashes: Map<String, String>
    ) -> Result<(), Error> {
        // Vérifier l'autorisation du fabricant
        manufacturer.require_auth();
        
        // Vérifier que le fabricant est un OEM enregistré
        Self::ensure_is_oem(&env, &manufacturer)?;
        
        // Vérifier que la pièce n'existe pas déjà
        let parts: Map<String, AeronauticPart> = env.storage().instance().get(&PARTS).unwrap_or(Map::new(&env));
        if parts.contains_key(uid.clone()) {
            return Err(Error::PartAlreadyExists);
        }
        
        // Créer la pièce
        let current_time = env.ledger().timestamp();
        let part = AeronauticPart {
            uid: uid.clone(),
            part_number,
            serial_number,
            manufacturer: manufacturer.clone(),
            date_of_manufacture: current_time,
            current_owner: manufacturer.clone(), // Le fabricant est le propriétaire initial
            status: PartStatus::Active,
            total_hours: 0,
            total_cycles: 0,
            last_updated: current_time,
            document_hashes,
        };
        
        // Ajouter la pièce au registre
        let mut updated_parts = parts.clone();
        updated_parts.set(uid.clone(), part);
        env.storage().instance().set(&PARTS, &updated_parts);
        
        // Prolonger la durée de vie du stockage
        env.storage().instance().extend_ttl(1000, 6_307_200);
        
        log!(&env, "Created new part: {} by manufacturer: {}", uid, manufacturer);
        Ok(())
    }
    
    // Obtenir les informations d'une pièce
    pub fn get_part(env: Env, uid: String) -> Result<AeronauticPart, Error> {
        let parts: Map<String, AeronauticPart> = env.storage().instance().get(&PARTS).unwrap_or(Map::new(&env));
        
        match parts.get(uid.clone()) {
            Some(part) => Ok(part),
            None => Err(Error::PartNotFound),
        }
    }
    
    // Transférer la propriété d'une pièce
    pub fn transfer_ownership(
        env: Env,
        current_owner: Address,
        new_owner: Address,
        uid: String
    ) -> Result<(), Error> {
        // Vérifier l'autorisation du propriétaire actuel
        current_owner.require_auth();
        
        // Récupérer les pièces
        let parts: Map<String, AeronauticPart> = env.storage().instance().get(&PARTS).unwrap_or(Map::new(&env));
        
        // Vérifier que la pièce existe
        let part = match parts.get(uid.clone()) {
            Some(p) => p,
            None => return Err(Error::PartNotFound),
        };
        
        // Vérifier que l'appelant est bien le propriétaire actuel
        if part.current_owner != current_owner {
            return Err(Error::NotAuthorized);
        }
        
        // Mettre à jour la propriété
        let current_time = env.ledger().timestamp();
        let mut updated_part = part.clone();
        updated_part.current_owner = new_owner.clone();
        updated_part.last_updated = current_time;
        
        // Mettre à jour le registre
        let mut updated_parts = parts.clone();
        updated_parts.set(uid.clone(), updated_part);
        env.storage().instance().set(&PARTS, &updated_parts);
        
        // Prolonger la durée de vie du stockage
        env.storage().instance().extend_ttl(1000, 6_307_200);
        
        log!(&env, "Transferred ownership of part: {} from: {} to: {}", uid, current_owner, new_owner);
        Ok(())
    }
    
    // Mettre à jour le statut d'une pièce (pour maintenance)
    pub fn update_part_status(
        env: Env,
        authorized_org: Address,
        uid: String,
        new_status: PartStatus,
        hours: u32,
        cycles: u32
    ) -> Result<(), Error> {
        // Vérifier l'autorisation de l'organisation
        authorized_org.require_auth();
        
        // Vérifier que l'organisation est un MRO ou le propriétaire
        Self::ensure_is_mro_or_owner(&env, &authorized_org, &uid)?;
        
        // Récupérer les pièces
        let parts: Map<String, AeronauticPart> = env.storage().instance().get(&PARTS).unwrap_or(Map::new(&env));
        
        // Vérifier que la pièce existe
        let part = match parts.get(uid.clone()) {
            Some(p) => p,
            None => return Err(Error::PartNotFound),
        };
        
        // Mettre à jour le statut et les compteurs
        let current_time = env.ledger().timestamp();
        let mut updated_part = part.clone();
        updated_part.status = new_status;
        updated_part.total_hours = hours;
        updated_part.total_cycles = cycles;
        updated_part.last_updated = current_time;
        
        // Mettre à jour le registre
        let mut updated_parts = parts.clone();
        updated_parts.set(uid.clone(), updated_part);
        env.storage().instance().set(&PARTS, &updated_parts);
        
        // Prolonger la durée de vie du stockage
        env.storage().instance().extend_ttl(1000, 6_307_200);
        
        log!(&env, "Updated status of part: {} to: {:?} by: {}", uid, new_status, authorized_org);
        Ok(())
    }
    
    // Ajouter un document à une pièce
    pub fn add_document(
        env: Env,
        authorized_org: Address,
        uid: String,
        document_name: String,
        document_hash: String
    ) -> Result<(), Error> {
        // Vérifier l'autorisation de l'organisation
        authorized_org.require_auth();
        
        // Vérifier que l'organisation est un MRO, OEM ou le propriétaire
        Self::ensure_can_add_document(&env, &authorized_org, &uid)?;
        
        // Récupérer les pièces
        let parts: Map<String, AeronauticPart> = env.storage().instance().get(&PARTS).unwrap_or(Map::new(&env));
        
        // Vérifier que la pièce existe
        let part = match parts.get(uid.clone()) {
            Some(p) => p,
            None => return Err(Error::PartNotFound),
        };
        
        // Ajouter le document
        let current_time = env.ledger().timestamp();
        let mut updated_part = part.clone();
        let mut updated_docs = updated_part.document_hashes.clone();
        updated_docs.set(document_name.clone(), document_hash.clone());
        updated_part.document_hashes = updated_docs;
        updated_part.last_updated = current_time;
        
        // Mettre à jour le registre
        let mut updated_parts = parts.clone();
        updated_parts.set(uid.clone(), updated_part);
        env.storage().instance().set(&PARTS, &updated_parts);
        
        // Prolonger la durée de vie du stockage
        env.storage().instance().extend_ttl(1000, 6_307_200);
        
        log!(
            &env, 
            "Added document: {} with hash: {} to part: {} by: {}", 
            document_name, document_hash, uid, authorized_org
        );
        Ok(())
    }
    
    // Fonctions d'aide privées
    
    // Vérifier si une adresse est un administrateur
    fn ensure_is_admin(env: &Env, address: &Address) -> Result<(), Error> {
        let admins: Vec<Address> = env.storage().instance().get(&ADMINS).unwrap_or(Vec::new(env));
        
        for admin in admins.iter() {
            if &admin == address {
                return Ok(());
            }
        }
        
        Err(Error::NotAuthorized)
    }
    
    // Vérifier si une adresse est un OEM enregistré
    fn ensure_is_oem(env: &Env, address: &Address) -> Result<(), Error> {
        let oem_orgs: Vec<Organization> = env.storage().instance().get(&OEM_ORGS).unwrap_or(Vec::new(env));
        
        for org in oem_orgs.iter() {
            if &org.id == address && org.active {
                return Ok(());
            }
        }
        
        Err(Error::NotAnOEM)
    }
    
    // Vérifier si une adresse est un MRO ou le propriétaire d'une pièce
    fn ensure_is_mro_or_owner(env: &Env, address: &Address, part_uid: &String) -> Result<(), Error> {
        // Vérifier si c'est un MRO
        let mro_orgs: Vec<Organization> = env.storage().instance().get(&MRO_ORGS).unwrap_or(Vec::new(env));
        let mut is_mro = false;
        
        for org in mro_orgs.iter() {
            if &org.id == address && org.active {
                is_mro = true;
                break;
            }
        }
        
        // Si ce n'est pas un MRO, vérifier si c'est le propriétaire
        if !is_mro {
            let parts: Map<String, AeronauticPart> = env.storage().instance().get(&PARTS).unwrap_or(Map::new(env));
            
            match parts.get(part_uid.clone()) {
                Some(part) => {
                    if &part.current_owner != address {
                        return Err(Error::NotAuthorized);
                    }
                },
                None => return Err(Error::PartNotFound),
            }
        }
        
        Ok(())
    }
    
    // Vérifier si une adresse peut ajouter un document (MRO, OEM ou propriétaire)
    fn ensure_can_add_document(env: &Env, address: &Address, part_uid: &String) -> Result<(), Error> {
        // Vérifier si c'est un MRO
        let mro_orgs: Vec<Organization> = env.storage().instance().get(&MRO_ORGS).unwrap_or(Vec::new(env));
        
        for org in mro_orgs.iter() {
            if &org.id == address && org.active {
                return Ok(());
            }
        }
        
        // Vérifier si c'est un OEM
        let oem_orgs: Vec<Organization> = env.storage().instance().get(&OEM_ORGS).unwrap_or(Vec::new(env));
        
        for org in oem_orgs.iter() {
            if &org.id == address && org.active {
                return Ok(());
            }
        }
        
        // Vérifier si c'est le propriétaire
        let parts: Map<String, AeronauticPart> = env.storage().instance().get(&PARTS).unwrap_or(Map::new(env));
        
        match parts.get(part_uid.clone()) {
            Some(part) => {
                if &part.current_owner == address {
                    return Ok(());
                }
            },
            None => return Err(Error::PartNotFound),
        }
        
        Err(Error::NotAuthorized)
    }

    // ===================================================
    // FONCTIONS DE LISTAGE SÉCURISÉES À AJOUTER AU CONTRAT
    // ===================================================
    // --------------------------------------------------
    // FONCTIONS POUR ADMINISTRATEURS SEULEMENT
    // --------------------------------------------------
    
    /// Obtenir TOUS les UIDs (ADMIN SEULEMENT)
    pub fn get_all_part_uids(env: Env, caller: Address) -> Result<Vec<String>, Error> {
        // Vérifier l'authentification
        caller.require_auth();
        
        // Vérifier que l'appelant est un administrateur
        Self::ensure_is_admin(&env, &caller)?;
        
        let parts: Map<String, AeronauticPart> = env.storage().instance().get(&PARTS).unwrap_or(Map::new(&env));
        let mut uids = Vec::new(&env);
        
        for key in parts.keys() {
            uids.push_back(key);
        }
        
        log!(&env, "Admin {} accessed all part UIDs (count: {})", caller, uids.len());
        Ok(uids)
    }

    /// Obtenir toutes les organisations (ADMIN SEULEMENT)
    pub fn get_all_organizations(env: Env, caller: Address) -> Result<(Vec<Organization>, Vec<Organization>), Error> {
        caller.require_auth();
        Self::ensure_is_admin(&env, &caller)?;
        
        let oem_orgs: Vec<Organization> = env.storage().instance().get(&OEM_ORGS).unwrap_or(Vec::new(&env));
        let mro_orgs: Vec<Organization> = env.storage().instance().get(&MRO_ORGS).unwrap_or(Vec::new(&env));
        
        log!(&env, "Admin {} accessed all organizations", caller);
        Ok((oem_orgs, mro_orgs))
    }

    /// Statistiques globales (ADMIN SEULEMENT)
    pub fn get_global_stats(env: Env, caller: Address) -> Result<(u32, u32, u32), Error> {
        caller.require_auth();
        Self::ensure_is_admin(&env, &caller)?;
        
        let parts: Map<String, AeronauticPart> = env.storage().instance().get(&PARTS).unwrap_or(Map::new(&env));
        let oem_orgs: Vec<Organization> = env.storage().instance().get(&OEM_ORGS).unwrap_or(Vec::new(&env));
        let mro_orgs: Vec<Organization> = env.storage().instance().get(&MRO_ORGS).unwrap_or(Vec::new(&env));
        
        let total_parts = parts.len();
        let total_oems = oem_orgs.len();
        let total_mros = mro_orgs.len();
        
        log!(&env, "Admin {} accessed global stats", caller);
        Ok((total_parts, total_oems, total_mros))
    }

    // --------------------------------------------------
    // FONCTIONS POUR PROPRIÉTAIRES DE PIÈCES
    // --------------------------------------------------
    
    /// Obtenir les UIDs des pièces dont on est propriétaire
    pub fn get_my_part_uids(env: Env, owner: Address) -> Result<Vec<String>, Error> {
        // Vérifier l'authentification
        owner.require_auth();
        
        let parts: Map<String, AeronauticPart> = env.storage().instance().get(&PARTS).unwrap_or(Map::new(&env));
        let mut my_uids = Vec::new(&env);
        
        // Filtrer les pièces appartenant à cet owner
        for (uid, part) in parts.iter() {
            if part.current_owner == owner {
                my_uids.push_back(uid);
            }
        }
        
        log!(&env, "Owner {} accessed their parts (count: {})", owner, my_uids.len());
        Ok(my_uids)
    }
    
    /// Obtenir les pièces qu'on a fabriquées (pour les OEMs)
    pub fn get_my_manufactured_parts(env: Env, manufacturer: Address) -> Result<Vec<String>, Error> {
        manufacturer.require_auth();
        
        // Vérifier que c'est bien un OEM enregistré
        Self::ensure_is_oem(&env, &manufacturer)?;
        
        let parts: Map<String, AeronauticPart> = env.storage().instance().get(&PARTS).unwrap_or(Map::new(&env));
        let mut manufactured_uids = Vec::new(&env);
        
        for (uid, part) in parts.iter() {
            if part.manufacturer == manufacturer {
                manufactured_uids.push_back(uid);
            }
        }
        
        log!(&env, "OEM {} accessed manufactured parts (count: {})", manufacturer, manufactured_uids.len());
        Ok(manufactured_uids)
    }


    // --------------------------------------------------
    // FONCTIONS POUR ORGANISATIONS AUTORISÉES
    // --------------------------------------------------
    
    /// Obtenir les pièces par statut (pour les MROs autorisés ou propriétaires)
    pub fn get_my_parts_by_status(
        env: Env, 
        caller: Address, 
        status: PartStatus
    ) -> Result<Vec<String>, Error> {
        caller.require_auth();
        
        let parts: Map<String, AeronauticPart> = env.storage().instance().get(&PARTS).unwrap_or(Map::new(&env));
        let mut matching_uids = Vec::new(&env);
        
        // Vérifier si c'est un admin (peut voir toutes les pièces)
        let is_admin = Self::ensure_is_admin(&env, &caller).is_ok();
        
        if is_admin {
            // Admin peut voir toutes les pièces avec ce statut
            for (uid, part) in parts.iter() {
                if part.status == status {
                    matching_uids.push_back(uid);
                }
            }
            log!(&env, "Admin {} accessed all parts with status {:?}", caller, status);
        } else {
            // Non-admin ne peut voir que ses propres pièces avec ce statut
            for (uid, part) in parts.iter() {
                if part.status == status && part.current_owner == caller {
                    matching_uids.push_back(uid);
                }
            }
            log!(&env, "User {} accessed their parts with status {:?} (count: {})", caller, status, matching_uids.len());
        }
        
        Ok(matching_uids)
    }
    
    /// Obtenir les pièces en maintenance pour un MRO
    pub fn get_parts_in_my_maintenance(env: Env, mro: Address) -> Result<Vec<String>, Error> {
        mro.require_auth();
        
        // Vérifier que c'est un MRO enregistré
        Self::ensure_is_mro(&env, &mro)?;
        
        let parts: Map<String, AeronauticPart> = env.storage().instance().get(&PARTS).unwrap_or(Map::new(&env));
        let mut maintenance_uids = Vec::new(&env);
        
        // Un MRO peut voir les pièces en maintenance qu'il a touchées
        // (logique métier : si le MRO a modifié la pièce récemment)
        // Ici, on simplifie en montrant toutes les pièces InMaintenance
        // Dans la vraie vie, il faudrait tracker qui fait quoi
        
        for (uid, part) in parts.iter() {
            if part.status == PartStatus::InMaintenance {
                maintenance_uids.push_back(uid);
            }
        }
        
        log!(&env, "MRO {} accessed parts in maintenance (count: {})", mro, maintenance_uids.len());
        Ok(maintenance_uids)
    }


    // ==========================================
    // FONCTIONS D'AIDE SUPPLÉMENTAIRES
    // ==========================================
    
    /// Vérifier si c'est un MRO enregistré (fonction d'aide réutilisable)
    fn ensure_is_mro(env: &Env, address: &Address) -> Result<(), Error> {
        let mro_orgs: Vec<Organization> = env.storage().instance().get(&MRO_ORGS).unwrap_or(Vec::new(env));
        
        for org in mro_orgs.iter() {
            if &org.id == address && org.active {
                return Ok(());
            }
        }
        
        Err(Error::OrgNotRegistered)
    }
    
    /// Obtenir des statistiques personnelles (nombre de pièces possédées)
    pub fn get_my_stats(env: Env, owner: Address) -> Result<(u32, u32, u32, u32), Error> {
        owner.require_auth();
        
        let parts: Map<String, AeronauticPart> = env.storage().instance().get(&PARTS).unwrap_or(Map::new(&env));
        
        let mut total_owned = 0u32;
        let mut active_parts = 0u32;
        let mut maintenance_parts = 0u32;
        let mut retired_parts = 0u32;
        
        for (_, part) in parts.iter() {
            if part.current_owner == owner {
                total_owned += 1;
                
                match part.status {
                    PartStatus::Active => active_parts += 1,
                    PartStatus::InMaintenance => maintenance_parts += 1,
                    PartStatus::Retired => retired_parts += 1,
                    PartStatus::Quarantined => {} // On peut ajouter si nécessaire
                }
            }
        }
        
        log!(&env, "User {} accessed personal stats", owner);
        Ok((total_owned, active_parts, maintenance_parts, retired_parts))
    }   


}

#[cfg(test)]
mod test;