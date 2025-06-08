# 🛩️ Aerochain - Système de Traçabilité des Pièces Aéronautiques

## 📋 Description

Aerochain est un contrat intelligent développé sur la blockchain Stellar/Soroban pour assurer la traçabilité complète des pièces aéronautiques. Ce système permet de suivre le cycle de vie complet des composants aéronautiques, depuis leur fabrication jusqu'à leur retrait, en garantissant l'authenticité et l'intégrité des données.

## 🎯 Objectifs

- **Traçabilité complète** : Suivi en temps réel de chaque pièce aéronautique
- **Sécurité renforcée** : Authentification des acteurs et vérification des autorisations
- **Conformité réglementaire** : Respect des standards aéronautiques internationaux
- **Transparence** : Historique immutable des opérations
- **Interopérabilité** : Interface standardisée pour tous les acteurs de l'industrie

## 🏗️ Architecture

### Types d'Organisations Supportées

- **OEM** (Original Equipment Manufacturer) : Fabricants de pièces originales
- **MRO** (Maintenance, Repair, and Operations) : Organismes de maintenance
- **Airlines** : Compagnies aériennes
- **Lessors** : Sociétés de leasing aéronautique
- **Distributors** : Distributeurs certifiés

### Structure des Données

#### Pièce Aéronautique (`AeronauticPart`)
```rust
pub struct AeronauticPart {
    pub uid: String,                    // Identifiant unique
    pub part_number: String,            // Numéro de pièce
    pub serial_number: String,          // Numéro de série
    pub manufacturer: Address,          // Fabricant
    pub date_of_manufacture: u64,       // Date de fabrication
    pub current_owner: Address,         // Propriétaire actuel
    pub status: PartStatus,             // Statut actuel
    pub total_hours: u32,               // Heures de vol totales
    pub total_cycles: u32,              // Cycles totaux
    pub last_updated: u64,              // Dernière mise à jour
    pub document_hashes: Map<String, String>, // Documents associés
}
```

#### Statuts des Pièces
- `Active` : Pièce en service
- `InMaintenance` : Pièce en maintenance
- `Retired` : Pièce retirée du service
- `Quarantined` : Pièce en quarantaine

## 🚀 Fonctionnalités Principales

### Administration
- **Initialisation du contrat** : Configuration initiale avec administrateur
- **Enregistrement d'organisations** : Ajout d'OEM et MRO certifiés
- **Gestion des autorisations** : Contrôle d'accès granulaire

### Gestion des Pièces
- **Création de pièces** : Enregistrement par les OEM autorisés
- **Transfert de propriété** : Changement de propriétaire sécurisé
- **Mise à jour du statut** : Modification du statut et des compteurs
- **Ajout de documents** : Association de documents certifiés

### Consultation et Traçabilité
- **Recherche de pièces** : Consultation par UID
- **Historique complet** : Suivi de toutes les modifications
- **Statistiques personnalisées** : Rapports pour chaque acteur
- **Listes filtrées** : Consultation par statut, propriétaire, etc.

## 🔐 Sécurité

### Authentification
- Vérification obligatoire de l'identité pour toutes les opérations
- Contrôle des autorisations par type d'organisation
- Protection contre les accès non autorisés

### Audit Trail
- Enregistrement de toutes les opérations dans les logs
- Horodatage précis de chaque modification
- Traçabilité complète des transferts de propriété

## 🛠️ Installation et Déploiement

### Prérequis
- Rust (version stable)
- Soroban CLI
- Stellar CLI
- Compte Stellar avec XLM pour les frais de transaction

### Compilation
```bash
# Cloner le repository
git clone <votre-repo>
cd aerochain

# Compiler le contrat
soroban contract build

# Déployer sur le testnet
soroban contract deploy \
    --wasm target/wasm32-unknown-unknown/release/aerochain.wasm \
    --network testnet
```

### Initialisation
```bash
# Initialiser le contrat avec un administrateur
soroban contract invoke \
    --id <CONTRACT_ID> \
    --network testnet \
    -- initialize \
    --admin <ADMIN_ADDRESS>
```

## 📖 Guide d'Utilisation

### Pour les Administrateurs

#### Enregistrer un OEM
```bash
soroban contract invoke \
    --id <CONTRACT_ID> \
    --network testnet \
    -- register_oem \
    --caller <ADMIN_ADDRESS> \
    --org_address <OEM_ADDRESS> \
    --name "Boeing Manufacturing" \
    --certificates '["AS9100", "ISO9001"]'
```

#### Enregistrer un MRO
```bash
soroban contract invoke \
    --id <CONTRACT_ID> \
    --network testnet \
    -- register_mro \
    --caller <ADMIN_ADDRESS> \
    --org_address <MRO_ADDRESS> \
    --name "Lufthansa Technik" \
    --certificates '["EASA Part 145"]'
```

### Pour les OEM

#### Créer une nouvelle pièce
```bash
soroban contract invoke \
    --id <CONTRACT_ID> \
    --network testnet \
    -- create_part \
    --manufacturer <OEM_ADDRESS> \
    --uid "AER-2024-001" \
    --part_number "737-ENG-001" \
    --serial_number "SN123456789" \
    --document_hashes '{"certificate": "hash123", "manual": "hash456"}'
```

### Pour tous les Acteurs

#### Consulter une pièce
```bash
soroban contract invoke \
    --id <CONTRACT_ID> \
    --network testnet \
    -- get_part \
    --uid "AER-2024-001"
```

#### Transférer la propriété
```bash
soroban contract invoke \
    --id <CONTRACT_ID> \
    --network testnet \
    -- transfer_ownership \
    --current_owner <CURRENT_OWNER> \
    --new_owner <NEW_OWNER> \
    --uid "AER-2024-001"
```

## 📊 Fonctions de Consultation

### Pour les Propriétaires
- `get_my_part_uids()` : Liste des pièces possédées
- `get_my_stats()` : Statistiques personnelles
- `get_my_parts_by_status()` : Pièces filtrées par statut

### Pour les OEM
- `get_my_manufactured_parts()` : Pièces fabriquées

### Pour les MRO
- `get_parts_in_my_maintenance()` : Pièces en maintenance

### Pour les Administrateurs
- `get_all_part_uids()` : Toutes les pièces du système
- `get_all_organizations()` : Toutes les organisations
- `get_global_stats()` : Statistiques globales

## 🔧 Structure du Projet

```
aerochain/
├── src/
│   ├── lib.rs              # Contrat principal
│   └── test.rs             # Tests unitaires
├── Cargo.toml              # Configuration Rust
├── README.md               # Documentation
└── .gitignore             # Fichiers à ignorer
```

## 🧪 Tests

```bash
# Exécuter les tests
cargo test

# Tests avec logs détaillés
cargo test -- --nocapture
```