#include <xapian.h>

#ifndef _XAPIAN_SHIM_H
#define _XAPIAN_SHIM_H

namespace shim {
  inline Xapian::RangeProcessor& date_range_processor_downcast(Xapian::DateRangeProcessor &rp) { return rp; }

  inline Xapian::Document document_copy(const Xapian::Document &doc) { return Xapian::Document(doc); }

  inline Xapian::MSet enquire_get_mset(
      const Xapian::Enquire &e, Xapian::doccount first,
      Xapian::doccount maxitems,  Xapian::doccount atleast
  ) { return e.get_mset(first, maxitems, atleast, nullptr, nullptr); }

  inline Xapian::MSetIterator mset_iterator_copy(const Xapian::MSetIterator &it) { return Xapian::MSetIterator(it); }
  inline void mset_iterator_decrement(Xapian::MSetIterator &it) { it--; }
  inline Xapian::docid mset_iterator_docid(const Xapian::MSetIterator &it) { return *it; }
  inline bool mset_iterator_eq(const Xapian::MSetIterator &a, const Xapian::MSetIterator &b) { return a == b; }
  inline void mset_iterator_increment(Xapian::MSetIterator &it) { it++; }

  inline Xapian::RangeProcessor& number_range_processor_downcast(Xapian::NumberRangeProcessor &rp) { return rp; }

  inline Xapian::PositionIterator position_iterator_copy(const Xapian::PositionIterator &it) { return Xapian::PositionIterator(it); }
  inline bool position_iterator_eq(const Xapian::PositionIterator &a, const Xapian::PositionIterator &b) { return a == b; }
  inline void position_iterator_increment(Xapian::PositionIterator &it) { it++; }
  inline Xapian::termpos position_iterator_position(const Xapian::PositionIterator &it) { return *it; }

  inline void query_parser_set_stopper(Xapian::QueryParser &qp, const Xapian::Stopper *stopper) { qp.set_stopper(stopper); }

  inline const Xapian::Stopper& simple_stopper_downcast(const Xapian::SimpleStopper &stopper) { return (const Xapian::Stopper&)(stopper); }
  inline bool simple_stopper_stop_at(const Xapian::SimpleStopper &stopper, const std::string &word) { return stopper(word); }

  inline std::string stemmer_stem(const Xapian::Stem &stem, const std::string &word) { return stem(word); }

  inline bool stopper_stop_at(const Xapian::Stopper &stopper, const std::string &word) { return stopper(word); }

  inline Xapian::TermIterator term_iterator_copy(const Xapian::TermIterator &it) { return Xapian::TermIterator(it); }
  inline bool term_iterator_eq(const Xapian::TermIterator &a, const Xapian::TermIterator &b) { return a == b; }
  inline void term_iterator_increment(Xapian::TermIterator &it) { it++; }
  inline std::string term_iterator_term(const Xapian::TermIterator &it) { return *it; }

  inline const Xapian::Database& writable_database_downcast(const Xapian::WritableDatabase &db) { return db; }
}

#endif
