//! CSL constants that describe entries, terms, and variables.

use std::fmt;
use std::num::IntErrorKind;
use std::str::FromStr;

use serde::{de, Deserialize, Deserializer, Serialize};

/// A CSL variable.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Variable {
    /// The set of variables with no other attributes.
    Standard(StandardVariable),
    /// Variables that can be formatted as numbers.
    Number(NumberVariable),
    /// Variables that can be formatted as dates.
    Date(DateVariable),
    /// Variables that can be formatted as names.
    Name(NameVariable),
}

impl Variable {
    /// Check if the variable starts with `number-of-` to control contextual
    /// label behavior.
    pub const fn is_number_of_variable(self) -> bool {
        if let Self::Number(v) = self {
            v.is_number_of_variable()
        } else {
            false
        }
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Standard(v) => v.fmt(f),
            Self::Number(v) => v.fmt(f),
            Self::Date(v) => v.fmt(f),
            Self::Name(v) => v.fmt(f),
        }
    }
}

/// The set of variables with no other attributes.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StandardVariable {
    /// Abstract of the item (e.g. the abstract of a journal article).
    Abstract,
    /// Short markup, decoration, or annotation to the item (e.g., to indicate
    /// items included in a review); For descriptive text (e.g., in an annotated
    /// bibliography), use note instead.
    Annote,
    /// Archive storing the item.
    Archive,
    /// Collection the item is part of within an archive.
    #[serde(rename = "archive_collection")]
    ArchiveCollection,
    /// Storage location within an archive (e.g. a box and folder number).
    #[serde(rename = "archive_location")]
    ArchiveLocation,
    /// Geographic location of the archive.
    ArchivePlace,
    /// Issuing or judicial authority (e.g. “USPTO” for a patent, “Fairfax
    /// Circuit Court” for a legal case).
    Authority,
    /// Call number (to locate the item in a library).
    CallNumber,
    /// Identifier of the item in the input data file (analogous to BibTeX
    /// entrykey); Use this variable to facilitate conversion between
    /// word-processor and plain-text writing systems; For an identifier intended
    /// as formatted output label for a citation (e.g. “Ferr78”), use
    /// citation-label instead.
    CitationKey,
    /// Label identifying the item in in-text citations of label styles (e.g.
    /// “Ferr78”); May be assigned by the CSL processor based on item metadata;
    /// For the identifier of the item in the input data file, use citation-key
    /// instead.
    CitationLabel,
    /// Title of the collection holding the item (e.g. the series title for a
    /// book; the lecture series title for a presentation).
    CollectionTitle,
    /// Title of the container holding the item (e.g. the book title for a book
    /// chapter, the journal title for a journal article; the album title for a
    /// recording; the session title for multi-part presentation at a
    /// conference).
    ContainerTitle,
    /// Short/abbreviated form of container-title; Deprecated; use
    /// variable="container-title" form="short" instead.
    ContainerTitleShort,
    /// Physical (e.g. size) or temporal (e.g. running time) dimensions of the
    /// item.
    Dimensions,
    /// Minor subdivision of a court with a jurisdiction for a legal item.
    Division,
    /// Digital Object Identifier (e.g. “10.1128/AEM.02591-07”).
    #[serde(rename = "DOI")]
    DOI,
    /// Deprecated legacy variant of event-title.
    Event,
    /// Name of the event related to the item (e.g. the conference name when
    /// citing a conference paper; the meeting where presentation was made).
    EventTitle,
    /// Geographic location of the event related to the item (e.g. “Amsterdam,
    /// The Netherlands”).
    EventPlace,
    /// Type, class, or subtype of the item (e.g. “Doctoral dissertation” for a
    /// PhD thesis; “NIH Publication” for an NIH technical report); Do not use
    /// for topical descriptions or categories (e.g. “adventure” for an
    /// adventure movie).
    Genre,
    /// International Standard Book Number (e.g. “978-3-8474-1017-1”).
    #[serde(rename = "ISBN")]
    ISBN,
    /// International Standard Serial Number.
    #[serde(rename = "ISSN")]
    ISSN,
    /// Geographic scope of relevance (e.g. “US” for a US patent; the court
    /// hearing a legal case).
    Jurisdiction,
    /// Keyword(s) or tag(s) attached to the item.
    Keyword,
    /// The language of the item.
    Language,
    /// The license information applicable to an item (e.g. the license an
    /// article or software is released under; the copyright information for an
    /// item; the classification status of a document).
    License,
    /// Description of the item’s format or medium (e.g. “CD”, “DVD”, “Album”,
    /// etc.).
    Medium,
    /// Descriptive text or notes about an item (e.g. in an annotated
    /// bibliography).
    Note,
    /// Original publisher, for items that have been republished by a different
    /// publisher.
    OriginalPublisher,
    /// Geographic location of the original publisher (e.g. “London, UK”).
    OriginalPublisherPlace,
    /// Title of the original version (e.g. “Война и мир”, the untranslated
    /// Russian title of “War and Peace”).
    OriginalTitle,
    /// Title of the specific part of an item being cited.
    PartTitle,
    /// PubMed Central reference number.
    #[serde(rename = "PMCID")]
    PMCID,
    /// PubMed reference number.
    #[serde(rename = "PMID")]
    PMID,
    /// Publisher.
    Publisher,
    /// Geographic location of the publisher.
    PublisherPlace,
    /// Resources related to the procedural history of a legal case or
    /// legislation; Can also be used to refer to the procedural history of
    /// other items (e.g. “Conference canceled” for a presentation accepted as a
    /// conference that was subsequently canceled; details of a retraction or
    /// correction notice).
    References,
    /// Type of the item being reviewed by the current item (e.g. book, film).
    ReviewedGenre,
    /// Title of the item reviewed by the current item.
    ReviewedTitle,
    /// Scale of e.g. a map or model.
    Scale,
    /// Source from whence the item originates (e.g. a library catalog or
    /// database).
    Source,
    /// Publication status of the item (e.g. “forthcoming”; “in press”; “advance
    /// online publication”; “retracted”).
    Status,
    /// Primary title of the item.
    Title,
    /// Short/abbreviated form of title; Deprecated; use variable="title"
    /// form="short" instead.
    TitleShort,
    /// Uniform Resource Locator (e.g.
    /// “https://aem.asm.org/cgi/content/full/74/9/2766”).
    #[allow(rustdoc::bare_urls)]
    #[serde(rename = "URL")]
    URL,
    /// Title of the volume of the item or container holding the item; Also use
    /// for titles of periodical special issues, special sections, and the like.
    VolumeTitle,
    /// Disambiguating year suffix in author-date styles (e.g. “a” in “Doe,
    /// 1999a”).
    YearSuffix,
}

impl From<StandardVariable> for Variable {
    fn from(value: StandardVariable) -> Self {
        Variable::Standard(value)
    }
}

impl fmt::Display for StandardVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Abstract => write!(f, "abstract"),
            Self::Annote => write!(f, "annote"),
            Self::Archive => write!(f, "archive"),
            Self::ArchiveCollection => write!(f, "archive_collection"),
            Self::ArchiveLocation => write!(f, "archive_location"),
            Self::ArchivePlace => write!(f, "archive_place"),
            Self::Authority => write!(f, "authority"),
            Self::CallNumber => write!(f, "call-number"),
            Self::CitationKey => write!(f, "citation-key"),
            Self::CitationLabel => write!(f, "citation-label"),
            Self::CollectionTitle => write!(f, "collection-title"),
            Self::ContainerTitle => write!(f, "container-title"),
            Self::ContainerTitleShort => write!(f, "container-title-short"),
            Self::Dimensions => write!(f, "dimensions"),
            Self::Division => write!(f, "division"),
            Self::DOI => write!(f, "DOI"),
            Self::Event => write!(f, "event"),
            Self::EventTitle => write!(f, "event-title"),
            Self::EventPlace => write!(f, "event-place"),
            Self::Genre => write!(f, "genre"),
            Self::ISBN => write!(f, "ISBN"),
            Self::ISSN => write!(f, "ISSN"),
            Self::Jurisdiction => write!(f, "jurisdiction"),
            Self::Keyword => write!(f, "keyword"),
            Self::Language => write!(f, "language"),
            Self::License => write!(f, "license"),
            Self::Medium => write!(f, "medium"),
            Self::Note => write!(f, "note"),
            Self::OriginalPublisher => write!(f, "original-publisher"),
            Self::OriginalPublisherPlace => write!(f, "original-publisher-place"),
            Self::OriginalTitle => write!(f, "original-title"),
            Self::PartTitle => write!(f, "part-title"),
            Self::PMCID => write!(f, "PMCID"),
            Self::PMID => write!(f, "PMID"),
            Self::Publisher => write!(f, "publisher"),
            Self::PublisherPlace => write!(f, "publisher-place"),
            Self::References => write!(f, "references"),
            Self::ReviewedGenre => write!(f, "reviewed-genre"),
            Self::ReviewedTitle => write!(f, "reviewed-title"),
            Self::Scale => write!(f, "scale"),
            Self::Source => write!(f, "source"),
            Self::Status => write!(f, "status"),
            Self::Title => write!(f, "title"),
            Self::TitleShort => write!(f, "title-short"),
            Self::URL => write!(f, "URL"),
            Self::VolumeTitle => write!(f, "volume-title"),
            Self::YearSuffix => write!(f, "year-suffix"),
        }
    }
}

/// Variables that can be formatted as numbers.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NumberVariable {
    /// Chapter number (e.g. chapter number in a book; track number on an
    /// album).
    ChapterNumber,
    /// Index (starting at 1) of the cited reference in the bibliography
    /// (generated by the CSL processor).
    CitationNumber,
    /// Number identifying the collection holding the item (e.g. the series
    /// number for a book).
    CollectionNumber,
    /// (Container) edition holding the item (e.g. “3” when citing a chapter in
    /// the third edition of a book).
    Edition,
    /// Number of a preceding note containing the first reference to the item;
    /// Assigned by the CSL processor; Empty in non-note-based styles or when
    /// the item hasn’t been cited in any preceding notes in a document.
    FirstReferenceNoteNumber,
    /// Issue number of the item or container holding the item (e.g. “5” when
    /// citing a journal article from journal volume 2, issue 5); Use
    /// volume-title for the title of the issue, if any.
    Issue,
    /// A cite-specific pinpointer within the item (e.g. a page number within a
    /// book, or a volume in a multi-volume work); Must be accompanied in the
    /// input data by a label indicating the locator type (see the Locators term
    /// list), which determines which term is rendered by cs:label when the
    /// locator variable is selected.
    Locator,
    /// Number identifying the item (e.g. a report number).
    Number,
    /// Total number of pages of the cited item.
    NumberOfPages,
    /// Total number of volumes, used when citing multi-volume books and such.
    NumberOfVolumes,
    /// Range of pages the item (e.g. a journal article) covers in a container
    /// (e.g. a journal issue).
    Page,
    /// First page of the range of pages the item (e.g. a journal article)
    /// covers in a container (e.g. a journal issue).
    PageFirst,
    /// Number of the specific part of the item being cited (e.g. part 2 of a
    /// journal article); Use part-title for the title of the part, if any.
    PartNumber,
    /// Printing number of the item or container holding the item.
    #[serde(alias = "printing")]
    PrintingNumber,
    /// Section of the item or container holding the item (e.g. “§2.0.1” for a
    /// law; “politics” for a newspaper article).
    Section,
    /// Supplement number of the item or container holding the item (e.g. for
    /// secondary legal items that are regularly updated between editions).
    SupplementNumber,
    /// Version of the item (e.g. “2.0.9” for a software program).
    Version,
    /// Volume number of the item (e.g. “2” when citing volume 2 of a book) or
    /// the container holding the item (e.g. “2” when citing a chapter from
    /// volume 2 of a book); Use volume-title for the title of the volume, if
    /// any.
    Volume,
}

impl fmt::Display for NumberVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChapterNumber => write!(f, "chapter-number"),
            Self::CitationNumber => write!(f, "citation-number"),
            Self::CollectionNumber => write!(f, "collection-number"),
            Self::Edition => write!(f, "edition"),
            Self::FirstReferenceNoteNumber => write!(f, "first-reference-note-number"),
            Self::Issue => write!(f, "issue"),
            Self::Locator => write!(f, "locator"),
            Self::Number => write!(f, "number"),
            Self::NumberOfPages => write!(f, "number-of-pages"),
            Self::NumberOfVolumes => write!(f, "number-of-volumes"),
            Self::Page => write!(f, "page"),
            Self::PageFirst => write!(f, "page-first"),
            Self::PartNumber => write!(f, "part-number"),
            Self::PrintingNumber => write!(f, "printing-number"),
            Self::Section => write!(f, "section"),
            Self::SupplementNumber => write!(f, "supplement-number"),
            Self::Version => write!(f, "version"),
            Self::Volume => write!(f, "volume"),
        }
    }
}

impl NumberVariable {
    /// Check if the variable starts with `number-of-` to control contextual
    /// label behavior.
    pub const fn is_number_of_variable(self) -> bool {
        matches!(self, Self::NumberOfPages | Self::NumberOfVolumes)
    }
}

impl From<NumberVariable> for Variable {
    fn from(value: NumberVariable) -> Self {
        Self::Number(value)
    }
}

impl From<NumberVariable> for Term {
    fn from(value: NumberVariable) -> Self {
        Self::NumberVariable(value)
    }
}

/// Variables that can be formatted as dates.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DateVariable {
    /// Date the item has been accessed.
    Accessed,
    /// Date the item was initially available (e.g. the online publication date
    /// of a journal article before its formal publication date; the date a
    /// treaty was made available for signing).
    AvailableDate,
    /// Date the event related to an item took place.
    EventDate,
    /// Date the item was issued/published.
    Issued,
    /// Issue date of the original version.
    OriginalDate,
    /// Date the item (e.g. a manuscript) was submitted for publication.
    Submitted,
}

impl From<DateVariable> for Variable {
    fn from(value: DateVariable) -> Self {
        Self::Date(value)
    }
}

impl fmt::Display for DateVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Accessed => write!(f, "accessed"),
            Self::AvailableDate => write!(f, "available-date"),
            Self::EventDate => write!(f, "event-date"),
            Self::Issued => write!(f, "issued"),
            Self::OriginalDate => write!(f, "original-date"),
            Self::Submitted => write!(f, "submitted"),
        }
    }
}

/// Variables that can be formatted as names.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NameVariable {
    /// Author.
    Author,
    /// The person leading the session containing a presentation (e.g. the
    /// organizer of the container-title of a speech).
    Chair,
    /// Editor of the collection holding the item (e.g. the series editor for a
    /// book).
    CollectionEditor,
    /// Person compiling or selecting material for an item from the works of
    /// various persons or bodies (e.g. for an anthology).
    Compiler,
    /// Composer (e.g. of a musical score).
    Composer,
    /// Author of the container holding the item (e.g. the book author for a
    /// book chapter).
    ContainerAuthor,
    /// A minor contributor to the item; typically cited using “with” before the
    /// name when listed in a bibliography.
    Contributor,
    /// Curator of an exhibit or collection (e.g. in a museum).
    Curator,
    /// Director (e.g. of a film).
    Director,
    /// Editor.
    Editor,
    /// Managing editor (“Directeur de la Publication” in French).
    EditorialDirector,
    /// Combined editor and translator of a work; The citation processory must
    /// be automatically generate if editor and translator variables are
    /// identical; May also be provided directly in item data.
    #[serde(rename = "editortranslator")]
    EditorTranslator,
    /// Executive producer (e.g. of a television series).
    ExecutiveProducer,
    /// Guest (e.g. on a TV show or podcast).
    Guest,
    /// Host (e.g. of a TV show or podcast).
    Host,
    /// Illustrator (e.g. of a children’s book or graphic novel).
    Illustrator,
    /// Interviewer (e.g. of an interview).
    Interviewer,
    /// Narrator (e.g. of an audio book).
    Narrator,
    /// Organizer of an event (e.g. organizer of a workshop or conference).
    Organizer,
    /// The original creator of a work (e.g. the form of the author name listed
    /// on the original version of a book; the historical author of a work; the
    /// original songwriter or performer for a musical piece; the original
    /// developer or programmer for a piece of software; the original author of
    /// an adapted work such as a book adapted into a screenplay).
    OriginalAuthor,
    /// Performer of an item (e.g. an actor appearing in a film; a muscian
    /// performing a piece of music).
    Performer,
    /// Producer (e.g. of a television or radio broadcast).
    Producer,
    /// Recipient (e.g. of a letter).
    Recipient,
    /// Author of the item reviewed by the current item.
    ReviewedAuthor,
    /// Writer of a script or screenplay (e.g. of a film).
    ScriptWriter,
    /// Creator of a series (e.g. of a television series).
    SeriesCreator,
    /// Translator.
    Translator,
}

impl From<NameVariable> for Variable {
    fn from(value: NameVariable) -> Self {
        Self::Name(value)
    }
}

impl From<NameVariable> for Term {
    fn from(value: NameVariable) -> Self {
        Self::NameVariable(value)
    }
}

impl fmt::Display for NameVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Author => write!(f, "author"),
            Self::Chair => write!(f, "chair"),
            Self::CollectionEditor => write!(f, "collection-editor"),
            Self::Compiler => write!(f, "compiler"),
            Self::Composer => write!(f, "composer"),
            Self::ContainerAuthor => write!(f, "container-author"),
            Self::Contributor => write!(f, "contributor"),
            Self::Curator => write!(f, "curator"),
            Self::Director => write!(f, "director"),
            Self::Editor => write!(f, "editor"),
            Self::EditorialDirector => write!(f, "editorial-director"),
            Self::EditorTranslator => write!(f, "editortranslator"),
            Self::ExecutiveProducer => write!(f, "executive-producer"),
            Self::Guest => write!(f, "guest"),
            Self::Host => write!(f, "host"),
            Self::Illustrator => write!(f, "illustrator"),
            Self::Interviewer => write!(f, "interviewer"),
            Self::Narrator => write!(f, "narrator"),
            Self::Organizer => write!(f, "organizer"),
            Self::OriginalAuthor => write!(f, "original-author"),
            Self::Performer => write!(f, "performer"),
            Self::Producer => write!(f, "producer"),
            Self::Recipient => write!(f, "recipient"),
            Self::ReviewedAuthor => write!(f, "reviewed-author"),
            Self::ScriptWriter => write!(f, "script-writer"),
            Self::SeriesCreator => write!(f, "series-creator"),
            Self::Translator => write!(f, "translator"),
        }
    }
}

/// Localizable terms.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Term {
    /// Kind of the cited item.
    Kind(Kind),
    /// Variables that can be formatted as names.
    NameVariable(NameVariable),
    /// Variables that can be formatted as numbers.
    NumberVariable(NumberVariable),
    /// A locator.
    Locator(Locator),
    /// Terms that are not defined via types, name or number variables.
    Other(OtherTerm),
}

impl Term {
    /// On which term this falls back if the given term is not available.
    pub const fn term_fallback(self) -> Self {
        match self {
            Self::Other(OtherTerm::LongOrdinal(i)) => Self::Other(OtherTerm::OrdinalN(i)),
            _ => self,
        }
    }

    /// Whether this is an ordinal term.
    pub const fn is_ordinal(self) -> bool {
        match self {
            Self::Other(other) => other.is_ordinal(),
            _ => false,
        }
    }

    /// Whether this is a numbered ordinal term.
    pub const fn is_n_ordinal(self) -> bool {
        match self {
            Self::Other(other) => other.is_n_ordinal(),
            _ => false,
        }
    }

    /// Whether this is a gendered term.
    pub const fn is_gendered(self) -> bool {
        if self.is_ordinal() {
            return true;
        };

        matches!(
            self,
            Self::Other(
                OtherTerm::Month01
                    | OtherTerm::Month02
                    | OtherTerm::Month03
                    | OtherTerm::Month04
                    | OtherTerm::Month05
                    | OtherTerm::Month06
                    | OtherTerm::Month07
                    | OtherTerm::Month08
                    | OtherTerm::Month09
                    | OtherTerm::Month10
                    | OtherTerm::Month11
                    | OtherTerm::Month12
            )
        )
    }

    /// Compare a term to another one. Return `true` if they are serialized the
    /// same.
    pub fn is_lexically_same(self, other: Self) -> bool {
        if self == other {
            return true;
        }

        let cmp = |a: Self, b: Self| {
            matches!(
                (a, b),
                (
                    Self::Locator(Locator::Issue),
                    Self::NumberVariable(NumberVariable::Issue),
                ) | (
                    Self::Locator(Locator::Page),
                    Self::NumberVariable(NumberVariable::Page),
                ) | (
                    Self::Locator(Locator::Section),
                    Self::NumberVariable(NumberVariable::Section),
                ) | (
                    Self::Locator(Locator::Volume),
                    Self::NumberVariable(NumberVariable::Volume),
                ) | (Self::Locator(Locator::Book), Self::Kind(Kind::Book))
                    | (Self::Locator(Locator::Chapter), Self::Kind(Kind::Chapter))
                    | (Self::Locator(Locator::Figure), Self::Kind(Kind::Figure))
            )
        };

        cmp(self, other) || cmp(other, self)
    }
}

/// Kind of the cited item.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]

pub enum Kind {
    /// A self-contained work made widely available but not published in a
    /// journal or other publication; Use for preprints, working papers, and
    /// similar works posted on a platform where some level of persistence or
    /// stewardship is expected (e.g. arXiv or other preprint repositories,
    /// working paper series); For unpublished works not made widely available
    /// or only hosted on personal websites, use manuscript.
    Article,
    /// An article published in an academic journal.
    ArticleJournal,
    /// An article published in a non-academic magazine.
    ArticleMagazine,
    /// An article published in a newspaper.
    ArticleNewspaper,
    /// A proposed piece of legislation.
    Bill,
    /// A book or similar work; Can be an authored book or an edited collection
    /// of self-contained chapters; Can be a physical book or an ebook; The
    /// format for an ebook may be specified using medium; Can be a
    /// single-volume work, a multivolume work, or one volume of a multivolume
    /// work; If a container-title is present, the item is interpreted as a book
    /// republished in a collection or anthology; Also used for whole conference
    /// proceedings volumes or exhibition catalogs by specifying event and
    /// related variables.
    Book,
    /// A recorded work broadcast over an electronic medium (e.g. a radio
    /// broadcast, a television show, a podcast); The type of broadcast may be
    /// specified using genre; If container-title is present, the item is
    /// interpreted as an episode contained within a larger broadcast series
    /// (e.g. an episode in a television show or an episode of a podcast).
    Broadcast,
    /// A part of a book cited separately from the book as a whole (e.g. a
    /// chapter in an edited book); Also used for introductions, forewords, and
    /// similar supplemental components of a book.
    Chapter,
    /// A classical or ancient work, sometimes cited using a common
    /// abbreviation.
    Classic,
    /// An archival collection in a museum or other institution.
    Collection,
    /// A data set or a similar collection of (mostly) raw data.
    Dataset,
    /// A catch-all category for items not belonging to other types; Use a more
    /// specific type when appropriate.
    Document,
    /// An entry in a database, directory, or catalog; For entries in a
    /// dictionary, use entry-dictionary; For entries in an encyclopedia, use
    /// entry-encyclopedia.
    Entry,
    /// An entry in a dictionary.
    EntryDictionary,
    /// An entry in an encyclopedia or similar reference work.
    EntryEncyclopedia,
    /// An organized event (e.g., an exhibition or conference); Use for direct
    /// citations to the event, rather than to works contained within an event
    /// (e.g. a presentation in a conference, a graphic in an exhibition) or
    /// based on an event (e.g. a paper-conference published in a proceedings,
    /// an exhibition catalog).
    Event,
    /// A illustration or representation of data, typically as part of a journal
    /// article or other larger work; May be in any format (e.g. image, video,
    /// audio recording, 3D model); The format of the item can be specified
    /// using medium.
    Figure,
    /// A still visual work; Can be used for artwork or other works (e.g.
    /// journalistic or historical photographs); Can be used for any still
    /// visual work (e.g. photographs, drawings, paintings, sculptures,
    /// clothing); The format of the item can be specified using medium.
    Graphic,
    /// A hearing by a government committee or transcript thereof.
    Hearing,
    /// An interview of a person; Also used for a recording or transcript of an
    /// interview; author is interpreted as the interviewee.
    Interview,
    /// A legal case.
    #[serde(rename = "legal_case")]
    LegalCase,
    /// A law or resolution enacted by a governing body.
    Legislation,
    /// An unpublished manuscript; Use for both modern unpublished works and
    /// classical manuscripts; For working papers, preprints, and similar works
    /// posted to a repository, use article.
    Manuscript,
    /// A geographic map.
    Map,
    /// A video or visual recording; If a container-title is present, the item
    /// is interpreted as a part contained within a larger compilation of
    /// recordings (e.g. a part of a multipart documentary)).
    #[serde(rename = "motion_picture")]
    MotionPicture,
    /// The printed score for a piece of music; For a live performance of the
    /// music, use performance; For recordings of the music, use song (for audio
    /// recordings) or motionPicture (for video recordings).
    #[serde(rename = "musical_score")]
    MusicalScore,
    /// A fragment, historical document, or other unusually-published or
    /// ephemeral work (e.g. a sales brochure).
    Pamphlet,
    /// A paper formally published in conference proceedings; For papers
    /// presented at a conference, but not published in a proceedings, use
    /// speech.
    PaperConference,
    /// A patent for an invention.
    Patent,
    /// A live performance of an artistic work; For non-artistic presentations,
    /// use speech; For recordings of a performance, use song or motionPicture.
    Performance,
    /// A full issue or run of issues in a periodical publication (e.g. a
    /// special issue of a journal).
    Periodical,
    /// Personal communications between multiple parties; May be unpublished
    /// (e.g. private correspondence between two researchers) or
    /// collected/published (e.g. a letter published in a collection).
    #[serde(rename = "personal_communication")]
    PersonalCommunication,
    /// A post on a online forum, social media platform, or similar platform;
    /// Also used for comments posted to online items.
    Post,
    /// A blog post.
    PostWeblog,
    /// An administrative order from any level of government.
    Regulation,
    /// A technical report, government report, white paper, brief, or similar
    /// work distributed by an institution; Also used for manuals and similar
    /// technical documentation (e.g. a software, instrument, or test manual);
    /// If a container-title is present, the item is interpreted as a chapter
    /// contained within a larger report.
    Report,
    /// A review of an item other than a book (e.g. a film review, posted peer
    /// review of an article); If reviewed-title is absent, title is taken to be
    /// the title of the reviewed item.
    Review,
    /// A review of a book; If reviewed-title is absent, title is taken to be
    /// the title of the reviewed book.
    ReviewBook,
    /// A computer program, app, or other piece of software.
    Software,
    /// An audio recording; Can be used for any audio recording (not only
    /// music); If a container-title is present, the item is interpreted as a
    /// track contained within a larger album or compilation of recordings.
    Song,
    /// A speech or other presentation (e.g. a paper, talk, poster, or symposium
    /// at a conference); Use genre to specify the type of presentation; Use
    /// event to indicate the event where the presentation was made (e.g. the
    /// conference name); Use container-title if the presentation is part of a
    /// larger session (e.g. a paper in a symposium); For papers published in
    /// conference proceedings, use paper-conference; For artistic performances,
    /// use performance.
    Speech,
    /// A technical standard or similar set of rules or norms.
    Standard,
    /// A thesis written to satisfy requirements for a degree; Use genre to
    /// specify the type of thesis.
    Thesis,
    /// A treaty agreement among political authorities.
    Treaty,
    /// A website or page on a website; Intended for sources which are
    /// intrinsically online; use a more specific type when appropriate (e.g.
    /// article-journal, post-weblog, report, entry); If a container-title is
    /// present, the item is interpreted as a page contained within a larger
    /// website.
    Webpage,
}

impl From<Kind> for Term {
    fn from(value: Kind) -> Self {
        Self::Kind(value)
    }
}

impl FromStr for Kind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "article" => Ok(Self::Article),
            "article-journal" => Ok(Self::ArticleJournal),
            "article-magazine" => Ok(Self::ArticleMagazine),
            "article-newspaper" => Ok(Self::ArticleNewspaper),
            "bill" => Ok(Self::Bill),
            "book" => Ok(Self::Book),
            "broadcast" => Ok(Self::Broadcast),
            "chapter" => Ok(Self::Chapter),
            "classic" => Ok(Self::Classic),
            "collection" => Ok(Self::Collection),
            "dataset" => Ok(Self::Dataset),
            "document" => Ok(Self::Document),
            "entry" => Ok(Self::Entry),
            "entry-dictionary" => Ok(Self::EntryDictionary),
            "entry-encyclopedia" => Ok(Self::EntryEncyclopedia),
            "event" => Ok(Self::Event),
            "figure" => Ok(Self::Figure),
            "graphic" => Ok(Self::Graphic),
            "hearing" => Ok(Self::Hearing),
            "interview" => Ok(Self::Interview),
            "legal_case" => Ok(Self::LegalCase),
            "legislation" => Ok(Self::Legislation),
            "manuscript" => Ok(Self::Manuscript),
            "map" => Ok(Self::Map),
            "motion_picture" => Ok(Self::MotionPicture),
            "musical_score" => Ok(Self::MusicalScore),
            "pamphlet" => Ok(Self::Pamphlet),
            "paper-conference" => Ok(Self::PaperConference),
            "patent" => Ok(Self::Patent),
            "performance" => Ok(Self::Performance),
            "periodical" => Ok(Self::Periodical),
            "personal_communication" => Ok(Self::PersonalCommunication),
            "post" => Ok(Self::Post),
            "post-weblog" => Ok(Self::PostWeblog),
            "regulation" => Ok(Self::Regulation),
            "report" => Ok(Self::Report),
            "review" => Ok(Self::Review),
            "review-book" => Ok(Self::ReviewBook),
            "software" => Ok(Self::Software),
            "song" => Ok(Self::Song),
            "speech" => Ok(Self::Speech),
            "standard" => Ok(Self::Standard),
            "thesis" => Ok(Self::Thesis),
            "treaty" => Ok(Self::Treaty),
            "webpage" => Ok(Self::Webpage),
            _ => Err(()),
        }
    }
}

/// A locator.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
#[allow(missing_docs)]
pub enum Locator {
    Act,
    Appendix,
    ArticleLocator,
    Book,
    Canon,
    Chapter,
    Column,
    Elocation,
    Equation,
    Figure,
    Folio,
    Issue,
    Line,
    Note,
    Opus,
    Page,
    Paragraph,
    Part,
    Rule,
    Scene,
    Section,
    #[serde(rename = "sub verbo", alias = "sub-verbo")]
    SubVerbo,
    Supplement,
    Table,
    Timestamp,
    Title,
    TitleLocator,
    Verse,
    Volume,
    /// The custom type is a `citationberg` addition. It will render nothing in
    /// the locator's `cs:label` element.
    #[serde(skip)]
    Custom,
}

impl From<Locator> for Term {
    fn from(value: Locator) -> Self {
        Self::Locator(value)
    }
}

impl FromStr for Locator {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "act" => Ok(Self::Act),
            "appendix" => Ok(Self::Appendix),
            "article-locator" => Ok(Self::ArticleLocator),
            "book" => Ok(Self::Book),
            "canon" => Ok(Self::Canon),
            "chapter" => Ok(Self::Chapter),
            "column" => Ok(Self::Column),
            "elocation" => Ok(Self::Elocation),
            "equation" => Ok(Self::Equation),
            "figure" => Ok(Self::Figure),
            "folio" => Ok(Self::Folio),
            "issue" => Ok(Self::Issue),
            "line" => Ok(Self::Line),
            "note" => Ok(Self::Note),
            "opus" => Ok(Self::Opus),
            "page" => Ok(Self::Page),
            "paragraph" => Ok(Self::Paragraph),
            "part" => Ok(Self::Part),
            "rule" => Ok(Self::Rule),
            "scene" => Ok(Self::Scene),
            "section" => Ok(Self::Section),
            "sub verbo" | "sub-verbo" => Ok(Self::SubVerbo),
            "supplement" => Ok(Self::Supplement),
            "table" => Ok(Self::Table),
            "timestamp" => Ok(Self::Timestamp),
            "title" => Ok(Self::Title),
            "title-locator" => Ok(Self::TitleLocator),
            "verse" => Ok(Self::Verse),
            "volume" => Ok(Self::Volume),
            _ => Err(()),
        }
    }
}

impl<'de> Deserialize<'de> for Locator {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;

        Self::from_str(&s).map_err(|_| de::Error::custom("invalid locator"))
    }
}

/// Terms that are not defined via types, name or number variables.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[allow(missing_docs)]
pub enum OtherTerm {
    // Months
    Month01,
    Month02,
    Month03,
    Month04,
    Month05,
    Month06,
    Month07,
    Month08,
    Month09,
    Month10,
    Month11,
    Month12,

    // Ordinals
    Ordinal,
    /// Between 0 and 99.
    OrdinalN(u8),
    /// Between 0 and 10.
    LongOrdinal(u8),

    // Punctuation
    OpenQuote,
    CloseQuote,
    OpenInnerQuote,
    CloseInnerQuote,
    PageRangeDelimiter,
    Colon,
    Comma,
    Semicolon,

    // Seasons
    Season01,
    Season02,
    Season03,
    Season04,

    // Disciplines
    Anthropology,
    Astronomy,
    Biology,
    Botany,
    Chemistry,
    Engineering,
    GenericBase,
    Geography,
    Geology,
    History,
    Humanities,
    Literature,
    Math,
    Medicine,
    Philosophy,
    Physics,
    Psychology,
    Sociology,
    Science,
    PoliticalScience,
    SocialScience,
    Theology,
    Zoology,

    // Miscellaneous
    Accessed,
    Ad,
    AdvanceOnlinePublication,
    Album,
    And,
    AndOthers,
    Anonymous,
    At,
    AudioRecording,
    AvailableAt,
    Bc,
    Bce,
    By,
    Ce,
    Circa,
    Cited,
    EtAl,
    Film,
    Forthcoming,
    From,
    Henceforth,
    Ibid,
    In,
    InPress,
    Internet,
    Interview,
    Letter,
    LocCit,
    NoDate,
    NoPlace,
    NoPublisher,
    On,
    Online,
    OpCit,
    OriginalWorkPublished,
    PersonalCommunication,
    Podcast,
    PodcastEpisode,
    Preprint,
    PresentedAt,
    RadioBroadcast,
    RadioSeries,
    RadioSeriesEpisode,
    Reference,
    Retrieved,
    ReviewOf,
    Scale,
    SpecialIssue,
    SpecialSection,
    TelevisionBroadcast,
    TelevisionSeries,
    TelevisionSeriesEpisode,
    Video,
    WorkingPaper,
}

impl OtherTerm {
    /// Whether this is a numbered ordinal term.
    pub const fn is_n_ordinal(self) -> bool {
        matches!(self, Self::OrdinalN(_) | Self::LongOrdinal(_))
    }

    /// Whether this is an ordinal term.
    pub const fn is_ordinal(self) -> bool {
        matches!(self, Self::Ordinal | Self::OrdinalN(_) | Self::LongOrdinal(_))
    }

    /// Get the month for a number between 0 and 11.
    pub const fn month(i: u8) -> Option<Self> {
        match i {
            0 => Some(Self::Month01),
            1 => Some(Self::Month02),
            2 => Some(Self::Month03),
            3 => Some(Self::Month04),
            4 => Some(Self::Month05),
            5 => Some(Self::Month06),
            6 => Some(Self::Month07),
            7 => Some(Self::Month08),
            8 => Some(Self::Month09),
            9 => Some(Self::Month10),
            10 => Some(Self::Month11),
            11 => Some(Self::Month12),
            _ => None,
        }
    }

    /// Get the season for a number between 0 and 3.
    pub const fn season(i: u8) -> Option<Self> {
        match i {
            0 => Some(Self::Season01),
            1 => Some(Self::Season02),
            2 => Some(Self::Season03),
            3 => Some(Self::Season04),
            _ => None,
        }
    }
}

impl FromStr for OtherTerm {
    type Err = TermConversionError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let parse_int = |s: &str| {
            s.parse::<u8>().map_err(|e| {
                if matches!(
                    e.kind(),
                    IntErrorKind::NegOverflow | IntErrorKind::PosOverflow
                ) {
                    TermConversionError::OutOfRange
                } else {
                    TermConversionError::Unknown
                }
            })
        };

        let month_prefix = "month-";
        let season_prefix = "season-";
        let ordinal_prefix = "ordinal-";
        let long_ordinal_prefix = "long-ordinal-";

        if let Some(month) = value.strip_prefix(month_prefix) {
            return match parse_int(month)? {
                1 => Ok(Self::Month01),
                2 => Ok(Self::Month02),
                3 => Ok(Self::Month03),
                4 => Ok(Self::Month04),
                5 => Ok(Self::Month05),
                6 => Ok(Self::Month06),
                7 => Ok(Self::Month07),
                8 => Ok(Self::Month08),
                9 => Ok(Self::Month09),
                10 => Ok(Self::Month10),
                11 => Ok(Self::Month11),
                12 => Ok(Self::Month12),
                _ => Err(TermConversionError::OutOfRange),
            };
        }

        if let Some(season) = value.strip_prefix(season_prefix) {
            return match parse_int(season)? {
                1 => Ok(Self::Season01),
                2 => Ok(Self::Season02),
                3 => Ok(Self::Season03),
                4 => Ok(Self::Season04),
                _ => Err(TermConversionError::OutOfRange),
            };
        }

        if let Some(ordinal) = value.strip_prefix(ordinal_prefix) {
            let ordinal = parse_int(ordinal)?;

            if ordinal > 99 {
                return Err(TermConversionError::OutOfRange);
            }

            return Ok(Self::OrdinalN(ordinal));
        }

        if let Some(long_ordinal) = value.strip_prefix(long_ordinal_prefix) {
            let ordinal = parse_int(long_ordinal)?;

            if ordinal > 10 {
                return Err(TermConversionError::OutOfRange);
            }

            return Ok(Self::LongOrdinal(ordinal));
        }

        match value {
            "ordinal" => Ok(Self::Ordinal),
            "open-quote" => Ok(Self::OpenQuote),
            "close-quote" => Ok(Self::CloseQuote),
            "open-inner-quote" => Ok(Self::OpenInnerQuote),
            "close-inner-quote" => Ok(Self::CloseInnerQuote),
            "page-range-delimiter" => Ok(Self::PageRangeDelimiter),
            "colon" => Ok(Self::Colon),
            "comma" => Ok(Self::Comma),
            "semicolon" => Ok(Self::Semicolon),
            "anthropology" => Ok(Self::Anthropology),
            "astronomy" => Ok(Self::Astronomy),
            "biology" => Ok(Self::Biology),
            "botany" => Ok(Self::Botany),
            "chemistry" => Ok(Self::Chemistry),
            "engineering" => Ok(Self::Engineering),
            "generic-base" => Ok(Self::GenericBase),
            "geography" => Ok(Self::Geography),
            "geology" => Ok(Self::Geology),
            "history" => Ok(Self::History),
            "humanities" => Ok(Self::Humanities),
            "literature" => Ok(Self::Literature),
            "math" => Ok(Self::Math),
            "medicine" => Ok(Self::Medicine),
            "philosophy" => Ok(Self::Philosophy),
            "physics" => Ok(Self::Physics),
            "psychology" => Ok(Self::Psychology),
            "sociology" => Ok(Self::Sociology),
            "science" => Ok(Self::Science),
            "political-science" | "political_science" => Ok(Self::PoliticalScience),
            "social-science" | "social_science" => Ok(Self::SocialScience),
            "theology" => Ok(Self::Theology),
            "zoology" => Ok(Self::Zoology),
            "accessed" => Ok(Self::Accessed),
            "ad" => Ok(Self::Ad),
            "advance-online-publication" => Ok(Self::AdvanceOnlinePublication),
            "album" => Ok(Self::Album),
            "and" => Ok(Self::And),
            "and-others" | "and others" => Ok(Self::AndOthers),
            "anonymous" => Ok(Self::Anonymous),
            "at" => Ok(Self::At),
            "audio-recording" => Ok(Self::AudioRecording),
            "available at" | "available-at" => Ok(Self::AvailableAt),
            "bc" => Ok(Self::Bc),
            "bce" => Ok(Self::Bce),
            "by" => Ok(Self::By),
            "ce" => Ok(Self::Ce),
            "circa" => Ok(Self::Circa),
            "cited" => Ok(Self::Cited),
            "et-al" => Ok(Self::EtAl),
            "film" => Ok(Self::Film),
            "forthcoming" => Ok(Self::Forthcoming),
            "from" => Ok(Self::From),
            "henceforth" => Ok(Self::Henceforth),
            "ibid" => Ok(Self::Ibid),
            "in" => Ok(Self::In),
            "in press" | "in-press" => Ok(Self::InPress),
            "internet" => Ok(Self::Internet),
            "interview" => Ok(Self::Interview),
            "letter" => Ok(Self::Letter),
            "loc-cit" => Ok(Self::LocCit),
            "no date" | "no-date" => Ok(Self::NoDate),
            "no-place" => Ok(Self::NoPlace),
            "no-publisher" => Ok(Self::NoPublisher),
            "on" => Ok(Self::On),
            "online" => Ok(Self::Online),
            "op-cit" => Ok(Self::OpCit),
            "original-work-published" => Ok(Self::OriginalWorkPublished),
            "personal-communication" => Ok(Self::PersonalCommunication),
            "podcast" => Ok(Self::Podcast),
            "podcast-episode" => Ok(Self::PodcastEpisode),
            "preprint" => Ok(Self::Preprint),
            "presented at" | "presented-at" => Ok(Self::PresentedAt),
            "radio-broadcast" => Ok(Self::RadioBroadcast),
            "radio-series" => Ok(Self::RadioSeries),
            "radio-series-episode" => Ok(Self::RadioSeriesEpisode),
            "reference" => Ok(Self::Reference),
            "retrieved" => Ok(Self::Retrieved),
            "review-of" => Ok(Self::ReviewOf),
            "scale" => Ok(Self::Scale),
            "special-issue" => Ok(Self::SpecialIssue),
            "special-section" => Ok(Self::SpecialSection),
            "television-broadcast" => Ok(Self::TelevisionBroadcast),
            "television-series" => Ok(Self::TelevisionSeries),
            "television-series-episode" => Ok(Self::TelevisionSeriesEpisode),
            "video" => Ok(Self::Video),
            "working-paper" => Ok(Self::WorkingPaper),
            _ => Err(TermConversionError::Unknown),
        }
    }
}

impl fmt::Display for OtherTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ordinal => write!(f, "ordinal"),
            Self::OpenQuote => write!(f, "open-quote"),
            Self::CloseQuote => write!(f, "close-quote"),
            Self::OpenInnerQuote => write!(f, "open-inner-quote"),
            Self::CloseInnerQuote => write!(f, "close-inner-quote"),
            Self::PageRangeDelimiter => write!(f, "page-range-delimiter"),
            Self::Colon => write!(f, "colon"),
            Self::Comma => write!(f, "comma"),
            Self::Semicolon => write!(f, "semicolon"),
            Self::Anthropology => write!(f, "anthropology"),
            Self::Astronomy => write!(f, "astronomy"),
            Self::Biology => write!(f, "biology"),
            Self::Botany => write!(f, "botany"),
            Self::Chemistry => write!(f, "chemistry"),
            Self::Engineering => write!(f, "engineering"),
            Self::GenericBase => write!(f, "generic-base"),
            Self::Geography => write!(f, "geography"),
            Self::Geology => write!(f, "geology"),
            Self::History => write!(f, "history"),
            Self::Humanities => write!(f, "humanities"),
            Self::Literature => write!(f, "literature"),
            Self::Math => write!(f, "math"),
            Self::Medicine => write!(f, "medicine"),
            Self::Philosophy => write!(f, "philosophy"),
            Self::Physics => write!(f, "physics"),
            Self::Psychology => write!(f, "psychology"),
            Self::Sociology => write!(f, "sociology"),
            Self::Science => write!(f, "science"),
            Self::PoliticalScience => write!(f, "political_science"),
            Self::SocialScience => write!(f, "social_science"),
            Self::Theology => write!(f, "theology"),
            Self::Zoology => write!(f, "zoology"),
            Self::Accessed => write!(f, "accessed"),
            Self::Ad => write!(f, "ad"),
            Self::AdvanceOnlinePublication => write!(f, "advance-online-publication"),
            Self::Album => write!(f, "album"),
            Self::And => write!(f, "and"),
            Self::AndOthers => write!(f, "and-others"),
            Self::Anonymous => write!(f, "anonymous"),
            Self::At => write!(f, "at"),
            Self::AudioRecording => write!(f, "audio-recording"),
            Self::AvailableAt => write!(f, "available at"),
            Self::Bc => write!(f, "bc"),
            Self::Bce => write!(f, "bce"),
            Self::By => write!(f, "by"),
            Self::Ce => write!(f, "ce"),
            Self::Circa => write!(f, "circa"),
            Self::Cited => write!(f, "cited"),
            Self::EtAl => write!(f, "et-al"),
            Self::Film => write!(f, "film"),
            Self::Forthcoming => write!(f, "forthcoming"),
            Self::From => write!(f, "from"),
            Self::Henceforth => write!(f, "henceforth"),
            Self::Ibid => write!(f, "ibid"),
            Self::In => write!(f, "in"),
            Self::InPress => write!(f, "in press"),
            Self::Internet => write!(f, "internet"),
            Self::Interview => write!(f, "interview"),
            Self::Letter => write!(f, "letter"),
            Self::LocCit => write!(f, "loc-cit"),
            Self::NoDate => write!(f, "no date"),
            Self::NoPlace => write!(f, "no-place"),
            Self::NoPublisher => write!(f, "no-publisher"),
            Self::On => write!(f, "on"),
            Self::Online => write!(f, "online"),
            Self::OpCit => write!(f, "op-cit"),
            Self::OriginalWorkPublished => write!(f, "original-work-published"),
            Self::PersonalCommunication => write!(f, "personal-communication"),
            Self::Podcast => write!(f, "podcast"),
            Self::PodcastEpisode => write!(f, "podcast-episode"),
            Self::Preprint => write!(f, "preprint"),
            Self::PresentedAt => write!(f, "presented at"),
            Self::RadioBroadcast => write!(f, "radio-broadcast"),
            Self::RadioSeries => write!(f, "radio-series"),
            Self::RadioSeriesEpisode => write!(f, "radio-series-episode"),
            Self::Reference => write!(f, "reference"),
            Self::Retrieved => write!(f, "retrieved"),
            Self::ReviewOf => write!(f, "review-of"),
            Self::Scale => write!(f, "scale"),
            Self::SpecialIssue => write!(f, "special-issue"),
            Self::SpecialSection => write!(f, "special-section"),
            Self::TelevisionBroadcast => write!(f, "television-broadcast"),
            Self::TelevisionSeries => write!(f, "television-series"),
            Self::TelevisionSeriesEpisode => write!(f, "television-series-episode"),
            Self::Video => write!(f, "video"),
            Self::WorkingPaper => write!(f, "working-paper"),
            Self::OrdinalN(i) => write!(f, "ordinal-{:02}", i),
            Self::LongOrdinal(i) => write!(f, "long-ordinal-{:02}", i),
            Self::Season01 => write!(f, "season-01"),
            Self::Season02 => write!(f, "season-02"),
            Self::Season03 => write!(f, "season-03"),
            Self::Season04 => write!(f, "season-04"),
            Self::Month01 => write!(f, "month-01"),
            Self::Month02 => write!(f, "month-02"),
            Self::Month03 => write!(f, "month-03"),
            Self::Month04 => write!(f, "month-04"),
            Self::Month05 => write!(f, "month-05"),
            Self::Month06 => write!(f, "month-06"),
            Self::Month07 => write!(f, "month-07"),
            Self::Month08 => write!(f, "month-08"),
            Self::Month09 => write!(f, "month-09"),
            Self::Month10 => write!(f, "month-10"),
            Self::Month11 => write!(f, "month-11"),
            Self::Month12 => write!(f, "month-12"),
        }
    }
}

impl<'de> Deserialize<'de> for OtherTerm {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

impl Serialize for OtherTerm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

/// An error that can occur when converting a string to a term.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TermConversionError {
    /// The month, season, ordinal, or long ordinal is out of range.
    OutOfRange,
    /// The term is unknown.
    Unknown,
}

impl fmt::Display for TermConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfRange => write!(f, "value out of range"),
            Self::Unknown => write!(f, "unknown term"),
        }
    }
}

impl From<OtherTerm> for Term {
    fn from(value: OtherTerm) -> Self {
        Self::Other(value)
    }
}
