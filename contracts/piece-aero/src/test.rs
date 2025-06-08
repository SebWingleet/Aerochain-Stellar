#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::{Address as _, Ledger}, vec, map};

#[test]
fn test_initialize_contract() {
    let env = Env::default();
    let contract_id = env.register(PartsRegistry, ());
    let client = PartsRegistryClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    
    // Initialiser le contrat
    let result = client.initialize(&admin);
    assert!(result.is_ok());
}

#[test]
fn test_register_oem() {
    let env = Env::default();
    let contract_id = env.register(PartsRegistry, ());
    let client = PartsRegistryClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    
    // Initialiser le contrat
    client.initialize(&admin).unwrap();
    
    // Créer une organisation OEM
    let oem_address = Address::generate(&env);
    let certificates = vec![&env, String::from_str(&env, "EASA.21G.0001")];
    
    // Enregistrer l'OEM avec l'admin
    let result = client.register_oem(&admin, &oem_address, &String::from_str(&env, "Safran"), &certificates);
    assert!(result.is_ok());
}

#[test]
fn test_create_part() {
    let env = Env::default();
    let contract_id = env.register(PartsRegistry, ());
    let client = PartsRegistryClient::new(&env, &contract_id);
    
    // Configurer le ledger avec un timestamp
    let timestamp = 1234567890;
    env.ledger().with_mut(|l| {
        l.timestamp = timestamp;
    });
    
    // Initialiser le contrat et les acteurs
    let admin = Address::generate(&env);
    client.initialize(&admin).unwrap();
    
    // Créer et enregistrer un OEM
    let oem_address = Address::generate(&env);
    let certificates = vec![&env, String::from_str(&env, "EASA.21G.0001")];
    client.register_oem(&admin, &oem_address, &String::from_str(&env, "Safran"), &certificates).unwrap();
    
    // Créer une pièce
    let uid = String::from_str(&env, "CFM56-5B4-123456");
    let part_number = String::from_str(&env, "CFM56-5B4");
    let serial_number = String::from_str(&env, "123456");
    
    // Ajouter des documents
    let mut docs = map![&env];
    docs.set(
        String::from_str(&env, "initial_cert"),
        String::from_str(&env, "1a2b3c4d5e6f7g8h9i0j")
    );
    
    // Créer la pièce avec l'OEM comme fabricant
    let result = client.create_part(&oem_address, &uid, &part_number, &serial_number, &docs);
    assert!(result.is_ok());
    
    // Vérifier que la pièce existe maintenant
    let part = client.get_part(&uid).unwrap();
    assert_eq!(part.uid, uid);
    assert_eq!(part.part_number, part_number);
    assert_eq!(part.serial_number, serial_number);
    assert_eq!(part.manufacturer, oem_address);
    assert_eq!(part.current_owner, oem_address); // Le fabricant est le propriétaire initial
    assert_eq!(part.total_hours, 0);
    assert_eq!(part.total_cycles, 0);
    assert_eq!(part.date_of_manufacture, timestamp);
}

#[test]
fn test_transfer_ownership() {
    let env = Env::default();
    let contract_id = env.register(PartsRegistry, ());
    let client = PartsRegistryClient::new(&env, &contract_id);
    
    // Configurer le ledger
    env.ledger().with_mut(|l| {
        l.timestamp = 1234567890;
    });
    
    // Initialiser le contrat et les acteurs
    let admin = Address::generate(&env);
    client.initialize(&admin).unwrap();
    
    // Créer et enregistrer un OEM
    let oem_address = Address::generate(&env);
    let certificates = vec![&env, String::from_str(&env, "EASA.21G.0001")];
    client.register_oem(&admin, &oem_address, &String::from_str(&env, "Safran"), &certificates).unwrap();
    
    // Créer une pièce
    let uid = String::from_str(&env, "CFM56-5B4-123456");
    let part_number = String::from_str(&env, "CFM56-5B4");
    let serial_number = String::from_str(&env, "123456");
    let docs = map![&env];
    
    // Créer la pièce avec l'OEM comme fabricant
    client.create_part(&oem_address, &uid, &part_number, &serial_number, &docs).unwrap();
    
    // Créer un nouveau propriétaire (compagnie aérienne)
    let airline_address = Address::generate(&env);
    
    // Transférer la propriété de l'OEM à la compagnie aérienne
    let result = client.transfer_ownership(&oem_address, &airline_address, &uid);
    assert!(result.is_ok());
    
    // Vérifier que le propriétaire a été mis à jour
    let part = client.get_part(&uid).unwrap();
    assert_eq!(part.current_owner, airline_address);
}

#[test]
#[should_panic(expected = "Error(NotAnOEM)")]
fn test_create_part_not_oem() {
    let env = Env::default();
    let contract_id = env.register(PartsRegistry, ());
    let client = PartsRegistryClient::new(&env, &contract_id);
    
    // Initialiser le contrat
    let admin = Address::generate(&env);
    client.initialize(&admin).unwrap();
    
    // Tenter de créer une pièce avec une adresse non-OEM
    let not_oem = Address::generate(&env);
    let uid = String::from_str(&env, "CFM56-5B4-123456");
    let part_number = String::from_str(&env, "CFM56-5B4");
    let serial_number = String::from_str(&env, "123456");
    let docs = map![&env];
    
    // Cette opération devrait échouer car l'adresse n'est pas un OEM enregistré
    client.create_part(&not_oem, &uid, &part_number, &serial_number, &docs);
}

#[test]
#[should_panic(expected = "Error(PartAlreadyExists)")]
fn test_create_duplicate_part() {
    let env = Env::default();
    let contract_id = env.register(PartsRegistry, ());
    let client = PartsRegistryClient::new(&env, &contract_id);
    
    // Initialiser le contrat et les acteurs
    let admin = Address::generate(&env);
    client.initialize(&admin).unwrap();
    
    // Créer et enregistrer un OEM
    let oem_address = Address::generate(&env);
    let certificates = vec![&env, String::from_str(&env, "EASA.21G.0001")];
    client.register_oem(&admin, &oem_address, &String::from_str(&env, "Safran"), &certificates).unwrap();
    
    // Données de la pièce
    let uid = String::from_str(&env, "CFM56-5B4-123456");
    let part_number = String::from_str(&env, "CFM56-5B4");
    let serial_number = String::from_str(&env, "123456");
    let docs = map![&env];
    
    // Créer la pièce une première fois
    client.create_part(&oem_address, &uid, &part_number, &serial_number, &docs).unwrap();
    
    // Tenter de créer la même pièce une seconde fois - devrait échouer
    client.create_part(&oem_address, &uid, &part_number, &serial_number, &docs);
}